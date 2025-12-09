use std::net::SocketAddr;

use auth_trait::Auth;
use log::info;
use ssl_trait::CertByHost;
use tokio::net::TcpListener;

use crate::{Mailer, Result};

pub async fn bind<A: Auth, M: Mailer>(
  listener: TcpListener,
  addr: SocketAddr,
  auth: A,
  mailer: std::sync::Arc<M>,
  ssl: impl CertByHost,
) -> Result<()> {
  let auth = std::sync::Arc::new(auth);
  info!("SMTP {addr} with implicit TLS");

  loop {
    // 接受新的客户端连接
    if let Ok((stream, addr)) = xerr::ok!(listener.accept().await) {
      let auth = auth.clone();
      let mailer = mailer.clone();
      let ssl = ssl.clone();

      // 为每个连接创建独立的task
      tokio::spawn(async move {
        // 设置15分钟连接超时
        let duration = std::time::Duration::from_secs(20 * 60);
        let result =
          tokio::time::timeout(duration, crate::conn::conn(addr, stream, auth, mailer, ssl)).await;
        if let Ok(result) = result {
          if let Err(e) = result {
            log::error!("❌ {}: {}", addr, e);
          }
        } else {
          log::error!("❌ {}: connection timed out", addr);
        }
      });
    }
  }
}
