use std::{borrow::Borrow, hash::Hash};

use expire_cache::Expire;
use papaya::HashSet;

pub struct ExpireSet<K: Hash + Eq + Send + Sync + Clone + 'static> {
  inner: Expire<HashSet<K>>,
}

impl<K: Hash + Eq + Send + Sync + Clone + 'static> ExpireSet<K> {
  pub fn new(expire: u64) -> Self {
    Self {
      inner: Expire::new(expire),
    }
  }

  pub fn insert(&self, key: K) {
    self.inner.insert(key, ());
  }

  pub fn contains<Q>(&self, key: &Q) -> bool
  where
    K: Borrow<Q>,
    Q: Hash + Eq + ?Sized,
  {
    self.inner.get(key).is_some()
  }
}
