use std::{borrow::Borrow, hash::Hash};

use papaya::HashMap;

use crate::Map;

#[cfg(feature = "get_or_init")]
impl<K, V> crate::GetOrInit for HashMap<K, V>
where
  K: Eq + Hash + Send + Sync + Clone + 'static,
  V: Send + Sync + Clone + 'static,
{
  fn get_or_init<'a, Q, E>(
    &'a self,
    key: &Q,
    func: impl FnOnce(&Q) -> Result<V, E>,
  ) -> Result<Self::RefVal<'a>, E>
  where
    K: Borrow<Q>,
    Q: ToOwned<Owned = K> + Hash + Eq + ?Sized,
  {
    let guard = self.guard();
    if let Some(v) = self.get(key, &guard) {
      return Ok(v.clone());
    }
    let v = func(key)?;
    self.insert(key.to_owned(), v.clone(), &guard);
    Ok(v)
  }
}

impl<K, V> Map for HashMap<K, V>
where
  K: Eq + Hash + Send + Sync + Clone + 'static,
  V: Send + Sync + Clone + 'static,
{
  type Key = K;
  type Val = V;
  type RefVal<'a> = V;

  fn clear(&self) {
    self.pin().clear();
  }

  fn insert(&self, key: Self::Key, val: Self::Val) {
    self.pin().insert(key, val);
  }

  fn get<'a, Q>(&'a self, key: &Q) -> Option<Self::RefVal<'a>>
  where
    K: Borrow<Q>,
    Q: Hash + Eq + ?Sized,
  {
    self.pin().get(key).cloned()
  }
}
