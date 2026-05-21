use std::{borrow::Borrow, hash::Hash, sync::atomic::Ordering};

use crate::Map;

pub trait GetOrInit: Map {
  fn get_or_init<'a, Q, E>(
    &'a self,
    key: &Q,
    func: impl FnOnce(&Q) -> Result<Self::Val, E>,
  ) -> Result<Self::RefVal<'a>, E>
  where
    Self::Key: Borrow<Q>,
    Q: ToOwned<Owned = Self::Key> + Hash + Eq + ?Sized;
}

impl<T: GetOrInit + 'static> crate::Expire<T> {
  pub fn get_or_init<'a, Q, E>(
    &'a self,
    key: &Q,
    func: impl FnOnce(&Q) -> Result<T::Val, E>,
  ) -> Result<T::RefVal<'a>, E>
  where
    T::Key: Borrow<Q>,
    Q: ToOwned<Owned = T::Key> + Hash + Eq + ?Sized,
  {
    let inner = self.inner();
    let pos = inner.pos.load(Ordering::Relaxed);
    if let Some(v) = inner.cache[pos].get(key) {
      return Ok(v);
    }
    if let Some(v) = inner.cache[1 - pos].get(key) {
      return Ok(v);
    }
    inner.cache[pos].get_or_init(key, func)
  }
}
