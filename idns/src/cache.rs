use expire_cache::Expire;
use papaya::HashMap;

use crate::{OptionVec, Parse, Query};

pub struct Cache<P: Parse + Clone> {
  pub cache: Expire<HashMap<String, OptionVec<P>>>,
}

impl<P: Parse + Clone> Cache<P> {
  pub fn new(timeout: u64) -> Self {
    Self {
      cache: Expire::new(timeout),
    }
  }

  pub async fn query<Q: Query>(
    &self,
    dns: &Q,
    name: impl AsRef<str>,
  ) -> Result<OptionVec<P>, Q::Error> {
    let name = name.as_ref();
    expire_cache::get_or_init_async!(self.cache, name, || dns.query(name))
  }
}
