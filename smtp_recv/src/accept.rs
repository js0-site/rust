use std::{future::Future, net::SocketAddr, time::Duration};

use log::info;
use ssl_trait::CertByHost;
use tokio::{net::TcpStream, time::timeout};

use crate::{Mailer, Result};

pub const TIMEOUT: u64 = 100;

pub trait Conn: Sync + Send + 'static {
  fn accept(
    &self,
    addr: SocketAddr,
    stream: TcpStream,
    mailer: impl Mailer,
    ssl: impl CertByHost,
  ) -> impl Future<Output = Result<()>> + Send;
}

pub async fn accept(
  conn: &impl Conn,
  stream: TcpStream,
  addr: SocketAddr,
  mailer: impl Mailer,
  ssl: impl CertByHost,
) {
  let duration = Duration::from_secs(TIMEOUT);
  let result = timeout(duration, conn.accept(addr, stream, mailer, ssl)).await;
  if let Ok(result) = result {
    if let Err(e) = result {
      info!("{addr}: {e}");
    }
  } else {
    info!("{addr}: connection timed out ( {TIMEOUT}s )");
  }
}
