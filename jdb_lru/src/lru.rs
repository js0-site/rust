//! LRU (Least Recently Used) cache
//! LRU（最近最少使用）缓存
//!
//! # Complexity
//! 复杂度
//!
//! - get: O(1)
//! - set: O(1)
//! - rm: O(1)
//!
//! Based on hashlink::LruCache with linked list for O(1) access order tracking.
//! 基于 hashlink::LruCache，用链表实现 O(1) 访问顺序跟踪。

use std::hash::Hash;

use hashlink::LruCache;

use crate::Cache;

/// LRU cache with fixed capacity
/// 固定容量的 LRU 缓存
///
/// Evicts least recently used items when full.
/// 满时淘汰最近最少使用的条目。
pub struct Lru<K: Hash + Eq, V>(pub LruCache<K, V>);

impl<K: Hash + Eq, V> Lru<K, V> {
  /// Create with capacity (min 1)
  /// 创建，指定容量（最小 1）
  #[inline(always)]
  pub fn new(cap: usize) -> Self {
    Self(LruCache::new(cap.max(1)))
  }
}

impl<K: Hash + Eq, V> Cache<K, V> for Lru<K, V> {
  #[inline(always)]
  fn get(&mut self, key: &K) -> Option<&V> {
    self.0.get(key)
  }

  #[inline(always)]
  fn set(&mut self, key: K, val: V) {
    self.0.insert(key, val);
  }

  #[inline(always)]
  fn rm(&mut self, key: &K) {
    self.0.remove(key);
  }
}
