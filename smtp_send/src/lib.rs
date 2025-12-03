use aok::Void;
use host::HOST;
use msgq::ReadGroup;
use sk_dkim::Sk;
mod send;

mod mail_parse;
use mail_parse::MailParse;

mod error;
pub use error::{Error, Result};

pub const R_SMTP: &str = "smtp";
pub const R_SEND: &str = "send";

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
