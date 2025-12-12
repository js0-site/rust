use std::{process, sync::Arc};

use auth_trait::Auth;
// use kill_port::kill_port;
use log::{error, info};
use parking_lot::RwLock;
use restart_signal::restart_signal;
use ssl_trait::CertByHost;
use tokio::{net::TcpListener, select};

use crate::{Mailer, Result, accept};

pub async fn bind<A: Auth, M: Mailer>(
  listener: TcpListener,
  auth: A,
  mailer: Arc<M>,
  ssl: impl CertByHost,
) -> Result<()> {
  let addr = listener.local_addr()?;
  info!("SMTP {addr} with implicit TLS");

  let lock = Arc::new(RwLock::new(()));

  let cancel_token = reload_self::listen()?;

  #[cfg(target_os = "linux")]
  {
    use sd_notify::NotifyState;
    let _ = sd_notify::notify(
      false,
      &[NotifyState::MainPid(std::process::id()), NotifyState::Ready],
    );
  }

  loop {
    select! {
      _ = cancel_token.cancelled() => {
        info!("reload self , shutdown pid {}", process::id());
        break;
      }
      _ = restart_signal() => {
        info!("recv signal , shutdown pid {}", process::id());
        break;
      },
      result = listener.accept() => {
        match result {
          Ok((stream, addr)) => {
            tokio::spawn({
              let lock = lock.clone();
              let auth = auth.clone();
              let mailer = mailer.clone();
              let ssl = ssl.clone();
              #[allow(clippy::await_holding_lock)]
              async move {
                let _guard = lock.read();
                accept(stream, addr, auth, mailer, ssl).await;
              }
            });
          }
          Err(e) => {
            error!("accept error: {e}");
            break;
          }
        }
      }
    }
  }

  let _guard = lock.write();

  Ok(())
}
