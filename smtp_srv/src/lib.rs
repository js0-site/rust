#![recursion_limit = "256"]

mod forward;
use forward::Forward;
mod cert;
pub use cert::Cert;
mod mailer;
pub use mailer::Mailer;
mod auth;
use auth::AuthKvrocks;
use fred::interfaces::KeysInterface;
use graceful_restart::CANCEL;
pub mod r;

pub async fn run() {
  log::info!("smtp_srv {}", env!("CARGO_PKG_VERSION"));
  let sk: Option<Vec<u8>> = xkv::R.get(r::DKIM_SK).await.ok().flatten();
  if sk.is_none() {
    log::warn!(
      "Global DKIM secret key ({}) is not configured in Kvrocks. Outgoing mail signing disabled.",
      r::DKIM_SK
    );
  }
  let mailer = Mailer::new(sk);

  if let Err(err) = smtp_recv::run(Forward, AuthKvrocks, mailer, Cert, CANCEL.clone()).await {
    log::error!("smtp_recv error: {err}");
  }
}
