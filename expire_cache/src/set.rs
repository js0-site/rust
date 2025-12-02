use std::hash::Hash;

use dashmap::DashSet;

use crate::Map;

impl<K> Map for DashSet<K>
where
  K: Eq + Hash + Clone + Send + Sync + 'static,
{
  type Key = K;
  type Val = ();

  fn clear(&self) {
    self.clear();
  }

  fn insert(&self, key: &Self::Key, _val: Self::Val) {
    self.insert(key.clone());
  }

  fn get(&self, key: &Self::Key) -> Option<Self::Val> {
    if self.contains(key) {
      Some(())
    } else {
      None
    }
  }
}
