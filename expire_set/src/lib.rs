use std::borrow::Borrow;

use dashmap::DashSet;
use expire_cache::Expire;

pub struct ExpireSet<K: std::hash::Hash + Eq + Send + Sync + 'static> {
  inner: Expire<DashSet<K>>,
}

impl<K: std::hash::Hash + Eq + Send + Sync + 'static> ExpireSet<K> {
  pub fn new(expire: u64) -> Self {
    Self {
      inner: Expire::new(expire),
    }
  }

  pub fn insert(&self, key: K) {
    self.inner.insert(key, ());
  }

  pub fn contains(&self, key: impl Borrow<K>) -> bool {
    self.inner.get(key).is_some()
  }
}
