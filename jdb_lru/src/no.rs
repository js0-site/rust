//! NoCache - zero overhead no-op cache
//! NoCache - 零开销空操作缓存
//!
//! # Complexity
//! 复杂度
//!
//! - get: O(1) - always returns None
//! - set: O(1) - no-op
//! - rm: O(1) - no-op
//!
//! Useful for disabling cache or GC-friendly code paths.
//! 用于禁用缓存或 GC 友好的代码路径。

use crate::Cache;

/// No-op cache, all operations do nothing
/// 空操作缓存，所有操作都不做任何事
pub struct NoCache;

impl<K, V> Cache<K, V> for NoCache {
  #[inline(always)]
  fn get(&mut self, _: &K) -> Option<&V> {
    None
  }

  #[inline(always)]
  fn set(&mut self, _: K, _: V) {}

  #[inline(always)]
  fn rm(&mut self, _: &K) {}
}
