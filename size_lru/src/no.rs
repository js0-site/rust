//! `NoCache` - zero overhead no-op cache
//! `NoCache` - 零开销空操作缓存

use std::{borrow::Borrow, hash::Hash};

use crate::SizeLru;

/// No-op cache
/// 空操作缓存
pub struct NoCache;

impl<K, V> SizeLru<K, V> for NoCache {
  type WithRm<Rm> = NoCache;

  fn with_on_rm<Rm>(_: usize, _: Rm) -> NoCache {
    NoCache
  }

  #[inline]
  fn get<Q>(&mut self, _: &Q) -> Option<&V>
  where
    K: Borrow<Q>,
    Q: Hash + Eq + ?Sized,
  {
    None
  }

  #[inline]
  fn peek<Q>(&self, _: &Q) -> Option<&V>
  where
    K: Borrow<Q>,
    Q: Hash + Eq + ?Sized,
  {
    None
  }

  #[inline(always)]
  fn set(&mut self, _: K, _: V, _: u32) {}

  #[inline(always)]
  fn rm<Q>(&mut self, _: &Q)
  where
    K: Borrow<Q>,
    Q: Hash + Eq + ?Sized,
  {
  }

  #[inline(always)]
  fn is_empty(&self) -> bool {
    true
  }

  #[inline(always)]
  fn len(&self) -> usize {
    0
  }
}
