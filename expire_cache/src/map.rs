use std::hash::Hash;

use dashmap::DashMap;

use crate::Map;

impl<K, V> Map for DashMap<K, V>
where
  K: Eq + Hash + Clone + Send + Sync + 'static,
  V: Clone + Send + Sync + 'static,
{
  type Key = K;
  type Val = V;

  fn clear(&self) {
    self.clear();
  }

  fn insert(&self, key: &Self::Key, val: Self::Val) {
    self.insert(key.clone(), val);
  }

  fn get(&self, key: &Self::Key) -> Option<Self::Val> {
    self.get(key).map(|v| v.value().clone())
  }
}
