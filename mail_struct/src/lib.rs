#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "send")]
mod send;

use std::collections::{HashMap, HashSet};

#[cfg(feature = "decode")]
use bitcode::Decode;
#[cfg(feature = "encode")]
use bitcode::Encode;
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
  pub host_user_li: HashMap<String, HashSet<String>>,
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

    let mut host_user_li: HashMap<_, HashSet<_>> = HashMap::new();
    for i in to_li {
      let i = i.as_ref();
      if let Some((user, host)) = xmail::norm_user_host(i) {
        host_user_li.entry(host).or_default().insert(user);
      } else {
        log::info!("mail invalid: {i}")
      }
    }

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
