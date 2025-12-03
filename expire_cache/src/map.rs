use std::{borrow::Borrow, future::Future, hash::Hash, ops::Deref};

use dashmap::DashMap;
use sendptr::SendPtr;

use crate::Map;

pub struct RefVal<'a, K, V> {
  _mapref: dashmap::mapref::one::Ref<'a, K, V>,
  val: SendPtr<V>,
}

impl<'a, K, V> RefVal<'a, K, V>
where
  K: Eq + Hash,
{
  pub fn new(mapref: dashmap::mapref::one::Ref<'a, K, V>) -> Self {
    let val = SendPtr::new(mapref.value() as *const V);
    Self {
      _mapref: mapref,
      val,
    }
  }
}

impl<'a, K, V> Borrow<V> for RefVal<'a, K, V> {
  fn borrow(&self) -> &V {
    &self.val
  }
}

impl<'a, K, V> AsRef<V> for RefVal<'a, K, V> {
  fn as_ref(&self) -> &V {
    &self.val
  }
}

impl<'a, K, V> Deref for RefVal<'a, K, V> {
  type Target = V;

  fn deref(&self) -> &Self::Target {
    &self.val
  }
}

#[cfg(feature = "get_or_init_async")]
impl<K, V> crate::GetOrInitAsync for DashMap<K, V>
where
  K: Clone + Eq + Hash + Send + Sync + 'static,
  V: Send + Sync + 'static,
{
  fn get_or_init_async<'a>(
    &'a self,
    key: &Self::Key,
    func: impl FnOnce(&Self::Key) -> std::pin::Pin<Box<dyn Future<Output = aok::Result<Self::Val>> + Send + 'a>> + Send + 'a,
  ) -> std::pin::Pin<Box<dyn Future<Output = aok::Result<Self::RefVal<'a>>> + Send + 'a>> {
    if let Some(r) = self.get(key) {
      return Box::pin(async move { Ok(RefVal::new(r)) });
    }
    let key = key.clone();
    let fut = func(&key);
    let self_ref = self;
    Box::pin(async move {
      let val = fut.await?;
      Ok(RefVal::new(
        self_ref.entry(key).or_insert(val).downgrade(),
      ))
    })
  }
}

#[cfg(feature = "get_or_init")]
impl<K, V> crate::GetOrInit for DashMap<K, V>
where
  K: Clone + Eq + Hash + Send + Sync + 'static,
  V: Send + Sync + 'static,
{
  fn get_or_init<'a>(
    &'a self,
    key: &Self::Key,
    func: impl Fn(&Self::Key) -> aok::Result<Self::Val>,
  ) -> aok::Result<Self::RefVal<'a>> {
    if let Some(r) = self.get(key) {
      return Ok(RefVal::new(r));
    }
    let val = func(key)?;
    Ok(RefVal::new(
      self.entry(key.clone()).or_insert(val).downgrade(),
    ))
  }
}

impl<K, V> Map for DashMap<K, V>
where
  K: Eq + Hash + Send + Sync + 'static,
  V: Send + Sync + 'static,
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
    Some(RefVal::new(mapref))
  }
}
