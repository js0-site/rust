use std::{
  fmt, io,
  net::SocketAddr,
  sync::{
    Arc, LazyLock,
    atomic::{AtomicU16, Ordering},
  },
  time::Duration,
};

use bytes::{BufMut, Bytes, BytesMut};
use idns::Answer;
use rustls::{crypto::ring::default_provider, pki_types::ServerName};
use tokio::{
  io::{AsyncReadExt, AsyncWriteExt},
  net::TcpStream,
  sync::RwLock,
  time::timeout,
};
use tokio_rustls::{TlsConnector, client::TlsStream};

use crate::{Error, HostIp, QType, Result};

/// DoT 默认端口 / DoT default port
pub const PORT: u16 = 853;

/// 查询超时 / Query timeout
const TIMEOUT: Duration = Duration::from_secs(9);

/// DNS 消息最大长度 / Max DNS message size
const MAX_MSG_LEN: usize = 65535;

/// DNS 消息最小长度（头部）/ Min DNS message size (header)
const MIN_MSG_LEN: usize = 12;

static CONF: LazyLock<Arc<rustls::ClientConfig>> = LazyLock::new(|| {
  let mut roots = rustls::RootCertStore::empty();
  roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

  Arc::new(
    rustls::ClientConfig::builder_with_provider(Arc::new(default_provider()))
      .with_safe_default_protocol_versions()
      .unwrap_or_else(|_| unreachable!())
      .with_root_certificates(roots)
      .with_no_client_auth(),
  )
});

/// DoT client with connection reuse / DoT 客户端，支持连接复用
pub struct Dot {
  pub server: HostIp,
  conn: RwLock<Option<TlsStream<TcpStream>>>,
  id: AtomicU16,
}

impl fmt::Debug for Dot {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Dot").field("server", &self.server).finish()
  }
}

impl Dot {
  pub fn new(server: HostIp) -> Self {
    Self {
      server,
      conn: RwLock::new(None),
      id: AtomicU16::new(1),
    }
  }

  fn next_id(&self) -> u16 {
    self.id.fetch_add(1, Ordering::Relaxed)
  }

  /// Get or create connection / 获取或创建连接
  async fn conn(&self) -> Result<TlsStream<TcpStream>> {
    // Try to take existing connection
    let existing = self.conn.write().await.take();
    if let Some(stream) = existing {
      return Ok(stream);
    }
    self.dial().await
  }

  /// Return connection for reuse / 归还连接以复用
  async fn return_conn(&self, stream: TlsStream<TcpStream>) {
    *self.conn.write().await = Some(stream);
  }

  async fn dial(&self) -> Result<TlsStream<TcpStream>> {
    let socket = TcpStream::connect(SocketAddr::new(self.server.ip, PORT)).await?;
    socket.set_nodelay(true)?;

    let connector = TlsConnector::from(CONF.clone());
    let name: ServerName<'_> = self
      .server
      .host
      .as_str()
      .try_into()
      .map_err(|_| Error::InvalidAddress(self.server.host.to_string()))?;

    timeout(TIMEOUT, connector.connect(name.to_owned(), socket))
      .await
      .map_err(|_| Error::Timeout)?
      .map_err(|e| Error::Io(io::Error::other(e)))
  }

  /// Execute DNS query / 执行 DNS 查询
  pub async fn query(&self, domain: &str, qtype: QType) -> Result<Option<Vec<Answer>>> {
    let mut stream = self.conn().await?;
    match self.send(&mut stream, domain, qtype).await {
      Ok(r) => {
        self.return_conn(stream).await;
        Ok(r)
      }
      Err(e) => {
        // Connection broken, don't return it
        Err(e)
      }
    }
  }

  async fn send(
    &self,
    stream: &mut TlsStream<TcpStream>,
    domain: &str,
    qtype: QType,
  ) -> Result<Option<Vec<Answer>>> {
    let id = self.next_id();
    let msg = dns_parse::build(id, domain, qtype as u16);

    // Send with 2-octet length prefix (RFC 7858)
    let mut buf = BytesMut::with_capacity(2 + msg.len());
    buf.put_u16(msg.len() as u16);
    buf.put_slice(&msg);

    stream.write_all(&buf).await?;
    stream.flush().await?;

    // Read response length
    let mut len_buf = [0u8; 2];
    timeout(TIMEOUT, stream.read_exact(&mut len_buf))
      .await
      .map_err(|_| Error::Timeout)??;

    let len = u16::from_be_bytes(len_buf) as usize;
    if !(MIN_MSG_LEN..=MAX_MSG_LEN).contains(&len) {
      return Err(Error::InvalidLength);
    }

    // Read response body
    let mut resp = vec![0u8; len];
    timeout(TIMEOUT, stream.read_exact(&mut resp))
      .await
      .map_err(|_| Error::Timeout)??;

    // Verify response ID matches request
    let resp_id = u16::from_be_bytes([resp[0], resp[1]]);
    if resp_id != id {
      return Err(Error::IdMismatch);
    }

    let answers = dns_parse::parse(Bytes::from(resp))?;
    Ok(if answers.is_empty() {
      None
    } else {
      Some(answers)
    })
  }
}
