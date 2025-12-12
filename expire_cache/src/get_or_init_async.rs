use std::{borrow::Borrow, future::Future, sync::atomic::Ordering};

use crate::Map;

pub trait AsyncInit {
  type Error;
  type Key;
  type Val;

  fn init(key: &Self::Key) -> impl Future<Output = Result<Self::Val, Self::Error>>;
}

pub trait GetOrInitAsync: Map {
  fn get_or_init_async<'a, I: AsyncInit<Key = Self::Key, Val = Self::Val>>(
    &'a self,
    key: &Self::Key,
  ) -> impl Future<Output = Result<Self::RefVal<'a>, I::Error>>;
}

impl<T: GetOrInitAsync + 'static> crate::Expire<T> {
  pub async fn get_or_init_async<'a, I: AsyncInit<Key = T::Key, Val = T::Val>>(
    &'a self,
    key: impl Borrow<T::Key> + Send,
  ) -> Result<T::RefVal<'a>, I::Error> {
    let inner = unsafe { &*self.inner };
    let key_ref = key.borrow();
    let pos = inner.pos.load(Ordering::Relaxed);
    if let Some(v) = inner.cache[1 - pos].get(key_ref) {
      return Ok(v);
    }
    inner.cache[pos].get_or_init_async::<I>(key_ref).await
  }
}
