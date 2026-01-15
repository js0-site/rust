//! Size-aware cache library with multiple implementations
//!
//! This library provides a common trait for size-aware caches with different eviction strategies.
//!
//! # Features
//!
//! - `lhd`: LHD (Least Hit Density) cache implementation
//! - `no`: `NoCache` - zero overhead no-op
//!
//! 大小感知缓存库，支持多种实现
//!
//! 本库为大小感知缓存提供通用 trait，支持不同的淘汰策略。
//!
//! # 特性

#![cfg_attr(docsrs, feature(doc_cfg))]

use std::{borrow::Borrow, hash::Hash};

/// Callback on entry removal/eviction
/// Called before actual removal or eviction, use `cache.peek(key)` to get value
///
/// # Design Rationale
///
/// Why callback only passes key, not value?
/// - Many use cases only need key (logging, counting, notifying external systems)
/// - If value not needed, avoids one memory access overhead
/// - When value needed, call `cache.peek(key)` to retrieve it
///
/// Why `&C` instead of `&mut C`?
/// - Prevents calling `get/rm/set` which would cause undefined behavior
/// - Only `peek` is safe during callback (read-only, no state mutation)
///
/// 条目删除/淘汰时的回调
/// 在实际删除或淘汰前调用，用 `cache.peek(key)` 获取值
///
/// # 设计原因
///
/// 为什么回调只传 key 而不传 value？
/// - 很多场景只需要 key（如日志、计数、通知外部系统）
/// - 若不需要 value，可避免一次内存访问开销
/// - 需要 value 时，调用 `cache.peek(key)` 即可获取
///
/// 为什么用 `&C` 而不是 `&mut C`？
/// - 防止调用 `get/rm/set`，这些会导致未定义行为
/// - 回调期间只有 `peek` 是安全的（只读，无状态变更）
pub trait OnRm<K, C> {
  fn call(&mut self, key: &K, cache: &C);
}

/// No-op callback (zero overhead)
pub struct NoOnRm;

//
/// 空回调（零开销）
impl<K, C> OnRm<K, C> for NoOnRm {
  #[inline(always)]
  fn call(&mut self, _: &K, _: &C) {}
}

/// Size-aware cache trait
pub trait SizeLru<K, V>: Sized {
  type WithRm<Rm>;

  fn new(max: usize) -> Self::WithRm<NoOnRm> {
    Self::with_on_rm(max, NoOnRm)
  }
  fn with_on_rm<Rm>(max: usize, on_rm: Rm) -> Self::WithRm<Rm>;
  fn get<Q>(&mut self, key: &Q) -> Option<&V>
  where
    K: Borrow<Q>,
    Q: Hash + Eq + ?Sized;
  /// Peek value without updating stats
  fn peek<Q>(&self, key: &Q) -> Option<&V>
  where
    K: Borrow<Q>,
    Q: Hash + Eq + ?Sized;
  fn set(&mut self, key: K, val: V, size: u32);
  fn rm<Q>(&mut self, key: &Q)
  where
    K: Borrow<Q>,
    Q: Hash + Eq + ?Sized;
  /// Check if cache is empty
  fn is_empty(&self) -> bool;
  /// Get entry count
  fn len(&self) -> usize;
}

//
/// 大小感知缓存 Trait
/// 查看值但不更新统计
/// 检查缓存是否为空
/// 获取条目数量
#[cfg(feature = "lhd")]
mod lhd;

#[cfg(feature = "no")]
mod no;

#[cfg(feature = "lhd")]
pub use lhd::Lhd;
#[cfg(feature = "no")]
pub use no::NoCache;
