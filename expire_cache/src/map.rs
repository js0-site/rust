use std::{borrow::Borrow, hash::Hash, ops::Deref};

use dashmap::DashMap;

pub struct RefVal<'a, K, V> {
  _mapref: dashmap::mapref::one::Ref<'a, K, V>,
  val: *const V,
}

impl<'a, K, V> Borrow<V> for RefVal<'a, K, V> {
  fn borrow(&self) -> &V {
    unsafe { &*self.val }
  }
}

impl<'a, K, V> AsRef<V> for RefVal<'a, K, V> {
  fn as_ref(&self) -> &V {
    unsafe { &*self.val }
  }
}

impl<'a, K, V> Deref for RefVal<'a, K, V> {
  type Target = V;

  fn deref(&self) -> &Self::Target {
    unsafe { &*self.val }
  }
}

use crate::Map;
impl<K, V> Map for DashMap<K, V>
where
  K: Eq + Hash + Clone + Send + Sync + 'static,
  V: Clone + Send + Sync + 'static,
{
  type Key = K;
  type Val = V;
  type RefVal<'a> = RefVal<'a, K, V>;

  fn clear(&self) {
    self.clear();
  }

  fn insert(&self, key: Self::Key, val: Self::Val) {
    self.insert(key, val);
  }

  fn get<'a>(&'a self, key: &Self::Key) -> Option<Self::RefVal<'a>> {
    let mapref = self.get(key.borrow())?;
    let val = mapref.value() as *const V;
    Some(RefVal {
      _mapref: mapref,
      val,
    })
  }
}
