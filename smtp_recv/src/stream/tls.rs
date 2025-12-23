use std::{borrow::Borrow, sync::Arc};

use rustls::ServerConfig;
use ssl_trait::CertByHost;
use tokio::{
  io::{BufReader, ReadHalf, WriteHalf},
  net::TcpStream,
};
use tokio_rustls::server::TlsStream as ServerTlsStream;

use super::ReaderWriter;
use crate::{Result, SmtpError};

/// TLS加密流
pub struct TlsStream {
  pub reader: BufReader<ReadHalf<ServerTlsStream<TcpStream>>>,
  pub writer: WriteHalf<ServerTlsStream<TcpStream>>,
}

impl ReaderWriter for TlsStream {
  type Reader = BufReader<ReadHalf<ServerTlsStream<TcpStream>>>;
  type Writer = WriteHalf<ServerTlsStream<TcpStream>>;

  fn reader(&mut self) -> &mut Self::Reader {
    &mut self.reader
  }

  fn writer(&mut self) -> &mut Self::Writer {
    &mut self.writer
  }
}

/// TLS握手辅助函数，用于统一处理TLS连接建立
///
/// 这个函数封装了常见的TLS握手流程：
/// 1. 启动惰性TLS接受器
/// 2. 提取SNI信息
/// 3. 获取对应的SSL证书
/// 4. 完成TLS握手
///
/// # 参数
/// - `stream`: TCP流
/// - `ssl`: SSL证书提供者
/// - `require_sni`: 是否要求SNI（465端口需要，25端口STARTTLS可选）
///
/// # 返回
/// - `Ok((tls_stream, sni_host))`: 成功时返回TLS流和SNI主机名
/// - `Err(error)`: 失败时返回错误
pub async fn perform_tls_handshake<S: CertByHost>(
  stream: TcpStream,
  ssl: &S,
  require_sni: bool,
) -> Result<(ServerTlsStream<TcpStream>, String)> {
  // 启动惰性TLS接受器，等待ClientHello
  let acceptor = tokio_rustls::LazyConfigAcceptor::new(Default::default(), stream);
  let start = acceptor.await?;
  let client_hello = start.client_hello();

  // 提取SNI（Server Name Indication）
  let sni_host = if require_sni {
    // 465端口等需要SNI的场景
    client_hello
      .server_name()
      .ok_or(SmtpError::NoSni)?
      .to_string()
  } else {
    // 25端口STARTTLS等可选SNI的场景
    client_hello
      .server_name()
      .map(|s| s.to_string())
      .unwrap_or_else(|| "localhost".to_string())
  };

  log::info!("TLS handshake for host: {}", sni_host);

  // 根据SNI获取对应的SSL证书
  let ssl_config = ssl
    .get(&sni_host)
    .await
    .map_err(|e| SmtpError::Certificate(e.into()))?
    .ok_or_else(|| SmtpError::NoCertificate(sni_host.clone()))?;

  // 构建TLS配置
  let ssl_ref = ssl_config.borrow();
  let config = ServerConfig::builder()
    .with_no_client_auth()
    .with_single_cert(ssl_ref.cert.clone(), ssl_ref.key.clone_key())?;

  // 完成TLS握手
  let tls_stream = start.into_stream(Arc::new(config)).await?;

  log::info!("TLS handshake completed for {}", sni_host);

  Ok((tls_stream, sni_host))
}
