use std::net::Ipv4Addr;

use crate::{Answer, QType};

#[cfg(feature = "cache")]
#[static_init::dynamic(lazy)]
pub static CACHE: crate::Cache<A> = crate::Cache::new(300);

#[derive(Debug, Clone)]
pub struct A {
  pub ip: Ipv4Addr,
  pub ttl: u64,
}

impl super::Parse for A {
  const QTYPE: QType = QType::A;

  fn li(answers: impl IntoIterator<Item = Answer>) -> Vec<Self> {
    answers
      .into_iter()
      .filter(|a| a.type_id == Self::QTYPE as u16)
      .filter_map(|a| {
        a.val.parse().ok().map(|ip| A {
          ip,
          ttl: a.ttl.into(),
        })
      })
      .collect()
  }
}
