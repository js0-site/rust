use std::{borrow::Borrow, sync::Arc};

use auth_trait::Auth;
use rustls::ServerConfig;
use ssl_trait::CertByHost;
use tokio::net::TcpStream;
use tokio_rustls::TlsStream;

use crate::{
  error::{Result, SmtpError},
  mailer::Mailer,
  session::Session,
};

/// 处理单个客户端连接
///
/// 工作流程：
/// 1. 进行TLS握手
/// 2. 从ClientHello中提取SNI
/// 3. 根据SNI获取对应的SSL证书
/// 4. 完成TLS握手
/// 5. 创建SMTP会话并开始处理命令
pub async fn conn<A: Auth, M: Mailer>(
  addr: std::net::SocketAddr,
  stream: TcpStream,
  auth: Arc<A>,
  mailer: Arc<M>,
  ssl: impl CertByHost,
) -> Result<()> {
  // 启动惰性TLS接受器，等待ClientHello
  let acceptor = tokio_rustls::LazyConfigAcceptor::new(Default::default(), stream);

  let start = acceptor.await?;

  let client_hello = start.client_hello();

  // 提取SNI（Server Name Indication）
  let host = client_hello
    .server_name()
    .ok_or(SmtpError::NoSni)?
    .to_string();

  log::info!("→ {host} {addr}");

  // 根据SNI获取对应的SSL证书
  let ssl_config = ssl
    .get(&host)
    .await
    .map_err(|e| SmtpError::Certificate(e.into()))?
    .ok_or_else(|| SmtpError::NoCertificate(host.clone()))?;

  // 构建TLS配置
  let ssl_ref = ssl_config.borrow();
  let config = ServerConfig::builder()
    .with_no_client_auth()
    .with_single_cert(ssl_ref.cert.clone(), ssl_ref.key.clone_key())?;

  // 完成TLS握手
  let tls_stream = start.into_stream(Arc::new(config)).await?;

  // 创建并运行SMTP会话
  Session::new(TlsStream::Server(tls_stream), auth, mailer, host)
    .run()
    .await
}
