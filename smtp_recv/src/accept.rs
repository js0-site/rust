use std::{net::SocketAddr, sync::Arc};

use auth_trait::Auth;
use ssl_trait::CertByHost;
use tokio::net::TcpStream;

use crate::{Mailer, conn};

pub async fn accept<A: Auth, M: Mailer>(
  stream: TcpStream,
  addr: SocketAddr,
  auth: A,
  mailer: Arc<M>,
  ssl: impl CertByHost,
) {
  tokio::spawn(async move {
    let duration = std::time::Duration::from_secs(20 * 60);
    let result = tokio::time::timeout(duration, conn(addr, stream, auth, mailer, ssl)).await;
    if let Ok(result) = result {
      if let Err(e) = result {
        log::error!("❌ {}: {}", addr, e);
      }
    } else {
      log::error!("❌ {}: connection timed out", addr);
    }
  });
}
