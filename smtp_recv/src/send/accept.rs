use std::net::SocketAddr;

use auth_trait::Auth;
use ssl_trait::CertByHost;
use tokio::net::TcpStream;
use tokio_rustls::TlsStream;

use super::session::Session;
use crate::{Mailer, Result, accept::Conn, stream::perform_tls_handshake};

/// 处理单个客户端连接
///
/// 工作流程：
/// 1. 进行TLS握手
/// 2. 从ClientHello中提取SNI
/// 3. 根据SNI获取对应的SSL证书
/// 4. 完成TLS握手
/// 5. 创建SMTP会话并开始处理命令

#[derive(Clone)]
pub struct Accept<A: Auth> {
  auth: A,
}

impl<A: Auth> Accept<A> {
  pub fn new(auth: A) -> Self {
    Self { auth }
  }
}

impl<A: Auth> Conn for Accept<A> {
  async fn accept(
    &self,
    addr: SocketAddr,
    stream: TcpStream,
    mailer: impl Mailer,
    ssl: impl CertByHost,
  ) -> Result<()> {
    // 使用统一的TLS握手函数（465端口需要SNI）
    let (tls_stream, host) = perform_tls_handshake(stream, &ssl, true).await?;

    log::info!("→ {host} {addr}");

    // 创建并运行SMTP会话
    Session::load(
      TlsStream::Server(tls_stream),
      self.auth.clone(),
      mailer,
      host,
    )?
    .run()
    .await
  }
}
