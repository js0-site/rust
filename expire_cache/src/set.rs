use std::{borrow::Borrow, hash::Hash};

use papaya::HashSet;

use crate::Map;

impl<K> Map for HashSet<K>
where
  K: Eq + Hash + Send + Sync + Clone + 'static,
{
  type Key = K;
  type Val = ();
  type RefVal<'a> = ();

  fn clear(&self) {
    self.pin().clear();
  }

  fn insert(&self, key: Self::Key, _val: Self::Val) {
    self.pin().insert(key);
  }

  fn get<'a, Q>(&'a self, key: &Q) -> Option<Self::RefVal<'a>>
  where
    K: Borrow<Q>,
    Q: Hash + Eq + ?Sized,
  {
    if self.pin().contains(key) {
      Some(())
    } else {
      None
    }
  }
}
