use std::{borrow::Borrow, sync::atomic::Ordering};

use crate::Map;

pub trait GetOrInit: Map {
  fn get_or_init<'a, E>(
    &'a self,
    key: &Self::Key,
    func: impl Fn(&Self::Key) -> Result<Self::Val, E>,
  ) -> Result<Self::RefVal<'a>, E>;
}

impl<T: GetOrInit + 'static> crate::Expire<T> {
  pub fn get_or_init<'a, E>(
    &'a self,
    key: impl Borrow<T::Key>,
    func: impl Fn(&T::Key) -> Result<T::Val, E>,
  ) -> Result<T::RefVal<'a>, E> {
    let key = key.borrow();
    let inner = unsafe { &*self.inner };
    let pos = inner.pos.load(Ordering::Relaxed);
    if let Some(v) = inner.cache[1 - pos].get(key) {
      return Ok(v);
    }
    inner.cache[pos].get_or_init(key, func)
  }
}
