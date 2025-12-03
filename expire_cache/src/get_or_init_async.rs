use std::{borrow::Borrow, future::Future, sync::atomic::Ordering};

use aok::Result;

use crate::Map;

pub trait GetOrInitAsync: Map {
  fn get_or_init_async<'a>(
    &'a self,
    key: &Self::Key,
    func: impl FnOnce(&Self::Key) -> std::pin::Pin<Box<dyn Future<Output = Result<Self::Val>> + Send + 'a>> + Send + 'a,
  ) -> std::pin::Pin<Box<dyn Future<Output = Result<Self::RefVal<'a>>> + Send + 'a>>;
}

impl<T: GetOrInitAsync + 'static> crate::Expire<T> {
  pub async fn get_or_init_async<'a>(
    &'a self,
    key: impl Borrow<T::Key>,
    func: impl FnOnce(&T::Key) -> std::pin::Pin<Box<dyn Future<Output = Result<T::Val>> + Send + 'a>> + Send + 'a,
  ) -> Result<T::RefVal<'a>> {
    let key = key.borrow();
    let inner = unsafe { &*self.inner };
    let pos = inner.pos.load(Ordering::Relaxed);
    if let Some(v) = inner.cache[1 - pos].get(key) {
      return Ok(v);
    }
    inner.cache[pos].get_or_init_async(key, func).await
  }
}
