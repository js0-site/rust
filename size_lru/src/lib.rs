//! Size-aware cache library with multiple implementations
//! 大小感知缓存库，支持多种实现
//!
//! This library provides a common trait for size-aware caches with different eviction strategies.
//! 本库为大小感知缓存提供通用 trait，支持不同的淘汰策略。
//!
//! # Features
//! # 特性
//!
//! - `lhd`: LHD (Least Hit Density) cache implementation
//! - `lhd`: LHD（最低命中密度）缓存实现
//! - `no`: `NoCache` - zero overhead no-op
//! - `no`: `NoCache` - 零开销空操作缓存

#![cfg_attr(docsrs, feature(doc_cfg))]

use std::{borrow::Borrow, hash::Hash};

/// Callback on entry removal/eviction
/// Called before actual removal or eviction, use `cache.peek(key)` to get value
/// 条目删除/淘汰时的回调
/// 在实际删除或淘汰前调用，用 `cache.peek(key)` 获取值
///
/// # Design Rationale
/// # 设计原因
///
/// Why callback only passes key, not value?
/// 为什么回调只传 key 而不传 value？
/// - Many use cases only need key (logging, counting, notifying external systems)
/// - 很多场景只需要 key（如日志、计数、通知外部系统）
/// - If value not needed, avoids one memory access overhead
/// - 若不需要 value，可避免一次内存访问开销
/// - When value needed, call `cache.peek(key)` to retrieve it
/// - 需要 value 时，调用 `cache.peek(key)` 即可获取
///
/// Why `&C` instead of `&mut C`?
/// 为什么用 `&C` 而不是 `&mut C`？
/// - Prevents calling `get/rm/set` which would cause undefined behavior
/// - 防止调用 `get/rm/set`，这些会导致未定义行为
/// - Only `peek` is safe during callback (read-only, no state mutation)
/// - 回调期间只有 `peek` 是安全的（只读，无状态变更）
pub trait OnRm<K, C> {
  fn call(&mut self, key: &K, cache: &C);
}

/// No-op callback (zero overhead)
/// 空回调（零开销）
pub struct NoOnRm;

impl<K, C> OnRm<K, C> for NoOnRm {
  #[inline(always)]
  fn call(&mut self, _: &K, _: &C) {}
}

/// Size-aware cache trait
/// 大小感知缓存 Trait
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
  /// 查看值但不更新统计
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
  /// 检查缓存是否为空
  fn is_empty(&self) -> bool;
  /// Get entry count
  /// 获取条目数量
  fn len(&self) -> usize;
}

pub mod error;
pub use error::{Error, Result};

#[cfg(feature = "lhd")]
mod lhd;

#[cfg(feature = "no")]
mod no;

#[cfg(feature = "wasm")]
pub mod wasm;

#[cfg(feature = "lhd")]
pub use lhd::Lhd;
#[cfg(feature = "no")]
pub use no::NoCache;
