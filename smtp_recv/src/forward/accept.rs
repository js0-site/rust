use std::net::SocketAddr;

use mail_forward::Forward;
use ssl_trait::CertByHost;
use tokio::net::TcpStream;

use super::session::Session;
use crate::{Mailer, Result, accept::Conn};

/// 25端口SMTP服务处理器
///
/// 工作流程：
/// 1. 接受明文TCP连接
/// 2. 支持STARTTLS升级到加密连接
/// 3. 不需要用户认证
/// 4. 接收邮件并通过Forward trait转发
/// 5. 主要用于接收来自其他邮件服务器的邮件

#[derive(Copy, Clone)]
pub struct Accept<T: Forward> {
  forward: T,
}

impl<T: Forward> Accept<T> {
  pub fn new(forward: T) -> Self {
    Self { forward }
  }
}

impl<T: Forward> Conn for Accept<T> {
  async fn accept(
    &self,
    addr: SocketAddr,
    stream: TcpStream,
    mailer: impl Mailer,
    ssl: impl CertByHost,
  ) -> Result<()> {
    log::info!("→ port 25 {}", addr);

    // 25端口直接使用明文连接开始，支持STARTTLS升级
    // 主机名将在STARTTLS时从SNI确定，这里使用空字符串作为占位符
    // 创建并运行转发会话，传递客户端IP用于SPF验证
    Session::new(stream, self.forward.clone(), mailer, ssl, addr.ip())
      .run()
      .await
  }
}
