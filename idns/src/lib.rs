#![cfg_attr(docsrs, feature(doc_cfg))]

use std::{error::Error, future::Future, result::Result as StdResult};

pub mod qtype;
pub use qtype::QType;

mod dns_race;
pub use dns_race::DnsRace;
pub use ip_set::{self, Ip4Range, Ip6Range, IpRange, IpSet, Ipv4Set, Ipv6Set, Range};

#[cfg(feature = "cache")]
mod cache;
#[cfg(feature = "cache")]
pub use cache::Cache;

mod parse;
pub use parse::Parse;
#[cfg(feature = "a")]
pub use parse::a::{self, A};
#[cfg(feature = "aaaa")]
pub use parse::aaaa::{self, Aaaa};
#[cfg(feature = "mx")]
pub use parse::mx::{self, Mx};
#[cfg(feature = "spf")]
pub use parse::spf::{self, Spf};

#[derive(Debug, Clone)]
pub struct Answer {
  pub name: String,
  pub val: String,
  pub type_id: u16,
  pub ttl: u32,
}

pub type OptionVec<T> = Option<Vec<T>>;

pub trait Query: Sync {
  type Error: Error + Send + Sync;

  fn query<P: Parse>(
    &self,
    name: &str,
  ) -> impl Future<Output = StdResult<OptionVec<P>, Self::Error>> + Send {
    async move { Ok(self.answer_li(P::QTYPE, name).await?.map(P::li)) }
  }

  fn answer_li(
    &self,
    qtype: QType,
    name: &str,
  ) -> impl Future<Output = StdResult<OptionVec<Answer>, Self::Error>> + Send;
}
