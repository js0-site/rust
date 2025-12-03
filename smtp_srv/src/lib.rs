use aok::{OK, Void};

mod cert;
pub use cert::Cert;
mod mailer;
use auth_env::AuthEnv;
pub use mailer::Mailer;

pub async fn run(port: u16) -> Void {
  smtp_recv::run(port, AuthEnv::load("SMTP")?, Mailer, Cert).await?;
  OK
}
