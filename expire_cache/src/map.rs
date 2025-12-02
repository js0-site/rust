use std::{borrow::Borrow, hash::Hash};

use dashmap::DashMap;

use crate::Map;
impl<K, V> Map for DashMap<K, V>
where
  K: Eq + Hash + Clone + Send + Sync + 'static,
  V: Clone + Send + Sync + 'static,
{
  type Key = K;
  type Val = V;
  type RefVal<'a> = dashmap::mapref::one::Ref<'a, K, V>;

  fn clear(&self) {
    self.clear();
  }

  fn insert(&self, key: Self::Key, val: Self::Val) {
    self.insert(key, val);
  }

  fn get<'a>(&'a self, key: &Self::Key) -> Option<Self::RefVal<'a>> {
    self.get(key.borrow())
  }
}
