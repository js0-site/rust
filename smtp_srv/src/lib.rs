#![recursion_limit = "256"]

mod forward;
use forward::Forward;
mod cert;
pub use cert::Cert;
mod mailer;
use auth_env::AuthEnv;
use graceful_restart::CANCEL;
pub use mailer::Mailer;
pub mod r;

pub async fn run() {
  if let Err(err) = smtp_recv::run(
    Forward,
    AuthEnv::load("SMTP").unwrap(),
    Mailer,
    Cert,
    CANCEL.clone(),
  )
  .await
  {
    log::error!("smtp_recv error: {err}");
  }
}
