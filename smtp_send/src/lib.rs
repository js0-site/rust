use aok::{OK, Void};
use host::HOST;
use log::info;
use mail_struct::Mail;
use msgq::{Kv, ReadGroup};
mod send;
use send::send;

mod error;
pub use error::{Error, Result};

pub const R_SMTP: &str = "smtp";
pub const R_SEND: &str = "send";

pub static ROOT_CERTS: std::sync::OnceLock<rustls::RootCertStore> = std::sync::OnceLock::new();

#[derive(Clone)]
pub struct MailParse;

impl msgq::Parse for MailParse {
  async fn run(&self, kv: &Kv) -> Void {
    for (user_id, mail) in kv {
      let user_id = intbin::bin_u64(user_id);
      let mail: Mail = bitcode::decode(mail)?;
      info!("{user_id} {} → {}", mail.sender, mail.to_li.join(" / "));
      send(mail).await?;
    }
    OK
  }

  async fn on_error(&self, kv: Kv, error: String) -> Void {
    for (user_id, mail) in kv {
      let user_id = intbin::bin_u64(user_id);
      let mail: Mail = bitcode::decode(&mail)?;
      info!(
        "{error} {user_id} {} → {}",
        mail.sender,
        mail.to_li.join(" / ")
      );
    }
    OK
  }
}

pub struct SmtpSend {
  read_group: ReadGroup<MailParse>,
}

impl SmtpSend {
  pub async fn run(&self) -> Void {
    self.read_group.run().await
  }
}

impl Default for SmtpSend {
  fn default() -> Self {
    SmtpSend {
      read_group: ReadGroup::new(
        MailParse,
        msgq::Conf::new("smtp", "send", &*HOST, 6, 300, 100, 3),
        // msgq::Conf::new("smtp", "send", &*HOST, 60, 300, 100, 3),
      ),
    }
  }
}
