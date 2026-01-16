#![cfg_attr(docsrs, feature(doc_cfg))]

mod host_user_li;
#[cfg(feature = "send")]
mod send;

#[cfg(feature = "decode")]
use bitcode::Decode;
#[cfg(feature = "encode")]
use bitcode::Encode;
pub use host_user_li::HostUserLi;
#[cfg(feature = "send")]
pub use send::MailMessage;

#[derive(Debug, Clone)]
pub struct UserMail {
  pub mail: Mail,
  pub user_id: u64,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "decode", derive(Decode))]
#[cfg_attr(feature = "encode", derive(Encode))]
pub struct Mail {
  pub sender_user: String,
  pub sender_host: String,
  pub host_user_li: HostUserLi,
  pub body: Vec<u8>,
}

impl Mail {
  pub fn new(
    sender: impl AsRef<str>,
    to_li: impl IntoIterator<Item = impl AsRef<str>>,
    body: impl Into<Vec<u8>>,
  ) -> Option<Self> {
    let sender = sender.as_ref();
    let (sender_user, sender_host) = match xmail::norm_user_host(sender) {
      Some((name, domain)) => (name, domain),
      None => {
        log::info!("sender invalid: {sender}");
        return None;
      }
    };

    let host_user_li = HostUserLi::from_iter(to_li);

    if host_user_li.is_empty() {
      return None;
    }

    Some(Self {
      sender_user,
      sender_host,
      host_user_li,
      body: body.into(),
    })
  }
}
