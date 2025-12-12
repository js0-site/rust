mod cert;
pub use cert::Cert;
mod mailer;
use auth_env::AuthEnv;
pub use mailer::Mailer;

pub async fn run(port: u16) {
  if let Err(err) = smtp_recv::run(port, AuthEnv::load("SMTP").unwrap(), Mailer, Cert).await {
    log::error!("smtp_recv error: {err}");
  }
}
