use std::{borrow::Borrow, hash::Hash};

use dashmap::DashSet;

use crate::Map;

impl<K> Map for DashSet<K>
where
  K: Eq + Hash + Send + Sync + 'static,
{
  type Key = K;
  type Val = ();
  type RefVal<'a> = ();

  fn clear(&self) {
    self.clear();
  }

  fn insert(&self, key: Self::Key, _val: Self::Val) {
    self.insert(key);
  }

  fn get<'a>(&'a self, key: &Self::Key) -> Option<Self::RefVal<'a>> {
    if self.contains(key.borrow()) {
      Some(())
    } else {
      None
    }
  }
}
