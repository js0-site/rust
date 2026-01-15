use graceful_restart::LOCK;
use log::{error, info};
use ssl_trait::CertByHost;
use tokio::{net::TcpListener, select};
use tokio_util::sync::CancellationToken;

use crate::{
  Mailer, Result,
  accept::{Conn, accept},
};

pub async fn bind<C: Conn>(
  conn: C,
  port: u16,
  mailer: impl Mailer,
  ssl: impl CertByHost,
  cancel_token: CancellationToken,
) -> Result<()> {
  drop::leak!(conn);
  let conn: &'static C = conn;

  {
    let listener = TcpListener::from_std(socket_port::listen(port)?)?;
    let local_addr = listener.local_addr()?;
    info!("listen {local_addr}");

    let _ = sys_notify::ready();

    loop {
      select! {
        _ = cancel_token.cancelled() => {
          break;
        },
        result = listener.accept() => {
          match result {
            Ok((stream, addr)) => {
              tokio::spawn({
                let mailer = mailer.clone();
                let ssl = ssl.clone();
                #[allow(clippy::await_holding_lock)]
                async move {
                  info!("{port} ← {addr}");
                  let _guard = LOCK.read();
                  accept(conn , stream, addr,  mailer, ssl).await;
                }
              });
            }
            Err(e) => {
              error!("port {port} accept error: {e}");
            }
          }
        }
      }
    }
  }

  let _guard = LOCK.write();

  Ok(())
}
