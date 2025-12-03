use aok::{OK, Void};
use host::HOST;
use log::info;
use mail_struct::Mail;
use msgq::{Kv, ReadGroup};
use sk_dkim::Sk;
mod send;
use send::send;

mod error;
pub use error::{Error, Result};

pub const R_SMTP: &str = "smtp";
pub const R_SEND: &str = "send";

pub static ROOT_CERTS: std::sync::OnceLock<rustls::RootCertStore> = std::sync::OnceLock::new();

pub struct MailParse {
  selector: String,
  sk: Sk,
}

impl msgq::Parse for MailParse {
  async fn run(&self, kv: &Kv, retry: u64) -> Void {
    for (user_id, mail) in kv {
      let user_id = intbin::bin_u64(user_id);
      let mail: Mail = bitcode::decode(mail)?;
      info!("{user_id} {} → {}", mail.sender, mail.to_li.join(" / "));
      send(mail, retry).await?;
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
  pub fn new(selector: impl Into<String>, sk: impl AsRef<[u8]>) -> Self {
    SmtpSend {
      read_group: ReadGroup::new(
        MailParse {
          selector: selector.into(),
          sk: Sk::new(sk),
        },
        msgq::Conf::new("smtp", "send", &*HOST, 6, 300, 100, 3),
        // msgq::Conf::new("smtp", "send", &*HOST, 60, 300, 100, 3),
      ),
    }
  }
  pub async fn run(&self) -> Void {
    self.read_group.run().await
  }
}
