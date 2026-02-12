use std::{
  net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
  sync::{Arc, LazyLock},
  time::Duration,
};

use bytes::{BufMut, Bytes, BytesMut};
use idns::Answer;
use quinn::{ClientConfig, Connection, Endpoint, TransportConfig, VarInt};
use rustls::pki_types::ServerName;
use tokio::sync::RwLock;

use crate::{Error, HostIp, QType, Result, parser};

/// DoQ 默认端口 / DoQ default port
pub const DOQ_PORT: u16 = 853;

/// 查询超时 / Query timeout
const TIMEOUT: Duration = Duration::from_secs(9);

/// DNS 消息最大长度 (RFC 9250 §4.6) / Max DNS message size
const MAX_DNS_MSG_LEN: usize = 65535;

const BIND_V4: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0);
const BIND_V6: SocketAddr = SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), 0);

static CONF: LazyLock<ClientConfig> = LazyLock::new(|| {
  let mut roots = rustls::RootCertStore::empty();
  roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

  let mut tls =
    rustls::ClientConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
      .with_safe_default_protocol_versions()
      .unwrap_or_else(|_| unreachable!())
      .with_root_certificates(roots)
      .with_no_client_auth();
  tls.alpn_protocols = vec![b"doq".to_vec()];

  let mut conf = ClientConfig::new(Arc::new(
    quinn::crypto::rustls::QuicClientConfig::try_from(tls).unwrap_or_else(|_| unreachable!()),
  ));

  let mut transport = TransportConfig::default();
  // 30s idle timeout / 30 秒空闲超时
  transport.max_idle_timeout(Some(VarInt::from_u32(30_000).into()));
  // Lower initial RTT estimate for faster retransmit / 降低初始 RTT 估计加快重传
  transport.initial_rtt(Duration::from_millis(100));
  // Keep-alive to prevent NAT/firewall timeout / 保活防止 NAT 超时
  transport.keep_alive_interval(Some(Duration::from_secs(10)));
  conf.transport_config(Arc::new(transport));
  conf
});

/// DoQ client with connection reuse / DoQ 客户端，支持连接复用
pub struct Doq {
  pub server: HostIp,
  conn: RwLock<Option<Connection>>,
}

impl std::fmt::Debug for Doq {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Doq").field("server", &self.server).finish()
  }
}

impl Doq {
  pub fn new(server: HostIp) -> Self {
    Self {
      server,
      conn: RwLock::new(None),
    }
  }

  /// Get or create connection / 获取或创建连接
  async fn conn(&self) -> Result<Connection> {
    let alive = |c: &Connection| c.close_reason().is_none();

    if let Some(c) = self.conn.read().await.as_ref().filter(|c| alive(c)) {
      return Ok(c.clone());
    }

    let mut guard = self.conn.write().await;
    if let Some(c) = guard.as_ref().filter(|c| alive(c)) {
      return Ok(c.clone());
    }

    let c = self.dial().await?;
    *guard = Some(c.clone());
    Ok(c)
  }

  async fn dial(&self) -> Result<Connection> {
    let bind = if self.server.ip.is_ipv6() {
      BIND_V6
    } else {
      BIND_V4
    };
    let mut ep = Endpoint::client(bind)?;
    ep.set_default_client_config(CONF.clone());

    let name: ServerName<'_> = self
      .server
      .host
      .as_str()
      .try_into()
      .map_err(|_| Error::InvalidAddress(self.server.host.to_string()))?;

    let addr = SocketAddr::new(self.server.ip, DOQ_PORT);
    tokio::time::timeout(TIMEOUT, ep.connect(addr, name.to_str().as_ref())?)
      .await
      .map_err(|_| Error::Timeout)?
      .map_err(Error::Connection)
  }

  /// Execute DNS query / 执行 DNS 查询
  pub async fn query(&self, domain: &str, qtype: QType) -> Result<Option<Vec<Answer>>> {
    let c = self.conn().await?;
    match Self::send(&c, domain, qtype).await {
      Ok(r) => Ok(r),
      Err(e) => {
        if matches!(e, Error::Connection(_) | Error::Io(_)) {
          *self.conn.write().await = None;
        }
        Err(e)
      }
    }
  }

  async fn send(c: &Connection, domain: &str, qtype: QType) -> Result<Option<Vec<Answer>>> {
    let (mut tx, mut rx) = c.open_bi().await.map_err(Error::Connection)?;

    // Build DNS query with Message ID = 0 (RFC 9250 §4.2.1)
    let msg = parser::build(domain, qtype as u16);

    // Send with 2-octet length prefix (RFC 9250 §4.2)
    let mut buf = BytesMut::with_capacity(2 + msg.len());
    buf.put_u16(msg.len() as u16);
    buf.put_slice(&msg);

    tx.write_all(&buf).await?;
    // Signal no more data will be sent (RFC 9250 §4.2)
    tx.finish()?;

    // Read response with max DNS message size limit
    let resp = tokio::time::timeout(TIMEOUT, rx.read_to_end(MAX_DNS_MSG_LEN))
      .await
      .map_err(|_| Error::Timeout)??;

    // Minimum response: 2-byte length + 12-byte header
    if resp.len() < 14 {
      return Ok(None);
    }

    let len = u16::from_be_bytes([resp[0], resp[1]]) as usize;
    if len != resp.len() - 2 {
      return Err(Error::LengthMismatch);
    }

    let answers = parser::parse(Bytes::from(resp).slice(2..))?;
    Ok(if answers.is_empty() {
      None
    } else {
      Some(answers)
    })
  }
}
