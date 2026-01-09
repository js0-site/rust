use std::net::Ipv6Addr;

use crate::{Answer, QType};

#[cfg(feature = "cache")]
#[static_init::dynamic(lazy)]
pub static CACHE: crate::Cache<Aaaa> = crate::Cache::new(300);

#[derive(Debug, Clone)]
pub struct Aaaa {
  pub ip: Ipv6Addr,
  pub ttl: u64,
}

impl super::Parse for Aaaa {
  const QTYPE: QType = QType::AAAA;

  fn li(answers: impl IntoIterator<Item = Answer>) -> Vec<Self> {
    answers
      .into_iter()
      .filter(|a| a.type_id == Self::QTYPE as u16)
      .filter_map(|a| {
        a.val.parse().ok().map(|ip| Aaaa {
          ip,
          ttl: a.ttl.into(),
        })
      })
      .collect()
  }
}
