//! Core cuckoo filter implementation.
//! 核心布谷鸟过滤器实现

use std::{hash::Hasher, mem::size_of};

use crate::buckets::Buckets;

/// Single cuckoo filter instance.
/// 单个布谷鸟过滤器实例
#[derive(Debug, Clone, bitcode::Encode, bitcode::Decode)]
pub struct Base {
  buckets: Buckets,
  max_kicks: usize,
  exceptional: Exceptional,
  count: usize,
}

impl Base {
  /// Create new cuckoo filter.
  /// 创建新的布谷鸟过滤器
  pub fn new(fp_bits: usize, entries: usize, items_hint: usize, max_kicks: usize) -> Self {
    let buckets_hint = items_hint.div_ceil(entries);
    let buckets = Buckets::new(fp_bits, entries, buckets_hint);
    Base {
      buckets,
      max_kicks,
      exceptional: Exceptional::new(),
      count: 0,
    }
  }

  /// Returns bits used.
  /// 返回使用的位数
  #[inline]
  pub fn bits(&self) -> u64 {
    self.buckets.bits() + self.exceptional.bits()
  }

  /// Returns item count.
  /// 返回元素数量
  #[inline]
  pub fn len(&self) -> usize {
    self.count
  }

  /// Returns capacity.
  /// 返回容量
  #[inline]
  pub fn capacity(&self) -> usize {
    self.buckets.total_entries() + self.exceptional.len()
  }

  /// Check if filter is empty.
  /// 检查过滤器是否为空
  #[inline]
  pub fn is_empty(&self) -> bool {
    self.count == 0
  }

  /// Check if filter is nearly full.
  /// 检查过滤器是否接近满
  #[inline]
  pub fn is_nearly_full(&self) -> bool {
    self.exceptional.has_kicked()
  }

  /// Check if item hash exists.
  /// 检查元素哈希是否存在
  #[inline]
  pub fn contains<H: Hasher + Clone>(&self, hasher: &H, hash: u64) -> bool {
    let (fp, i0) = self.buckets.fp_and_index(hash);
    if fp == 0 {
      let i1 = self.buckets.alt_index(hasher, i0, fp);
      return self.exceptional.contains(i0, i1, fp);
    }
    // Hot path: check i0 first, defer i1 calculation
    // 热路径：先检查 i0，延迟计算 i1
    if self.buckets.contains(i0, fp) {
      return true;
    }
    let i1 = self.buckets.alt_index(hasher, i0, fp);
    self.buckets.contains(i1, fp)
      || (!self.exceptional.is_empty() && self.exceptional.contains(i0, i1, fp))
  }

  /// Insert item hash.
  /// 插入元素哈希
  #[inline]
  pub fn insert<H: Hasher + Clone>(&mut self, hasher: &H, hash: u64) {
    let (fp, i0) = self.buckets.fp_and_index(hash);
    self.insert_fp(hasher, i0, fp);
  }

  /// Remove item hash.
  /// 移除元素哈希
  #[inline]
  pub fn remove<H: Hasher + Clone>(&mut self, hasher: &H, hash: u64) -> bool {
    let (fp, i0) = self.buckets.fp_and_index(hash);
    let i1 = self.buckets.alt_index(hasher, i0, fp);

    // Try remove in order: bucket i0 -> bucket i1 -> exceptional
    // 按顺序尝试移除：桶 i0 -> 桶 i1 -> exceptional
    let removed = if fp == 0 {
      self.exceptional.remove(i0, i1, fp)
    } else {
      self.buckets.remove(i0, fp)
        || self.buckets.remove(i1, fp)
        || (!self.exceptional.is_empty() && self.exceptional.remove(i0, i1, fp))
    };

    if removed {
      self.count = self.count.saturating_sub(1);
    }
    removed
  }

  /// Shrink filter to fit current items.
  /// 收缩过滤器以适应当前元素
  #[inline]
  pub fn shrink_to_fit<H: Hasher + Clone>(&mut self, hasher: &H) {
    let entries = self.buckets.entries_per_bucket();
    // Ensure at least 1 to avoid 0-size bucket
    // 确保至少为 1 以避免 0 大小的桶
    let items_hint = self.count.max(1);
    let shrunk_len = Buckets::required_buckets(items_hint.div_ceil(entries));
    if shrunk_len < self.buckets.len() {
      let mut shrunk = Base::new(
        self.buckets.fp_bitwidth(),
        entries,
        items_hint,
        self.max_kicks,
      );
      for (i, fp) in self.buckets.iter() {
        let shrunk_i = shrunk.buckets.index(i as u64);
        shrunk.insert_fp(hasher, shrunk_i, fp);
      }
      for &(fp, min_i) in &self.exceptional.0 {
        if fp == 0 {
          let shrunk_i0 = shrunk.buckets.index(min_i as u64);
          let shrunk_i1 = shrunk.buckets.alt_index(hasher, shrunk_i0, 0);
          shrunk.exceptional.insert(shrunk_i0, shrunk_i1, 0);
          shrunk.count += 1;
        } else {
          let shrunk_i = shrunk.buckets.index(min_i as u64);
          shrunk.insert_fp(hasher, shrunk_i, fp);
        }
      }
      *self = shrunk;
    }
    self.exceptional.shrink_to_fit();
  }

  /// Insert fingerprint at index.
  /// 在索引处插入指纹
  #[inline]
  fn insert_fp<H: Hasher + Clone>(&mut self, hasher: &H, i0: usize, mut fp: u64) {
    self.count += 1;

    if fp == 0 {
      let i1 = self.buckets.alt_index(hasher, i0, fp);
      self.exceptional.insert(i0, i1, 0);
      return;
    }

    // Try i0 first (most common case)
    // 先尝试 i0（最常见情况）
    if self.buckets.try_insert(i0, fp) {
      return;
    }

    // Compute i1 only when needed
    // 仅在需要时计算 i1
    let i1 = self.buckets.alt_index(hasher, i0, fp);
    if self.buckets.try_insert(i1, fp) {
      return;
    }

    // Boundary case: if max_kicks is 0, store directly in exceptional
    // 边界情况：若最大踢出次数为 0，直接存入异常区，避免错误随机交换
    if self.max_kicks == 0 {
      self.exceptional.insert(i0, i1, fp);
      return;
    }

    let mut i = if fastrand::bool() { i0 } else { i1 };
    let mut prev_i = i;
    for _ in 0..self.max_kicks {
      fp = self.buckets.random_swap(i, fp);
      prev_i = i;
      i = self.buckets.alt_index(hasher, i, fp);
      if self.buckets.try_insert(i, fp) {
        return;
      }
    }
    self.exceptional.insert(prev_i, i, fp);
  }
}

/// Storage for exceptional items (kicked out or zero fingerprint).
/// 异常元素存储（被踢出或零指纹）
#[derive(Debug, Clone, Default, bitcode::Encode, bitcode::Decode)]
struct Exceptional(Vec<(u64, usize)>);

impl Exceptional {
  fn new() -> Self {
    Exceptional(Vec::new())
  }

  #[inline]
  fn len(&self) -> usize {
    self.0.len()
  }

  #[inline]
  fn is_empty(&self) -> bool {
    self.0.is_empty()
  }

  #[inline]
  fn bits(&self) -> u64 {
    (size_of::<(u64, usize)>() * self.0.capacity()) as u64 * 8
  }

  #[inline]
  fn shrink_to_fit(&mut self) {
    self.0.shrink_to_fit();
  }

  /// Check if has kicked out entries (non-zero fp).
  /// 检查是否有被踢出的条目（非零 fp）。
  ///
  /// 注：`self.0` 按 `(fp, min(i0, i1))` 升序排列。
  /// 由于非零 `fp > 0`，任何非零条目均排在 `fp == 0` 的条目之后，
  /// 因此 `self.0.last()` 是否非零即为 O(1) 精确判断是否存在踢出条目。
  #[inline]
  fn has_kicked(&self) -> bool {
    self.0.last().is_some_and(|&(fp, _)| fp != 0)
  }

  /// Create key for binary search.
  /// 创建二分查找的键
  #[inline(always)]
  fn key(i0: usize, i1: usize, fp: u64) -> (u64, usize) {
    (fp, i0.min(i1))
  }

  #[inline]
  fn contains(&self, i0: usize, i1: usize, fp: u64) -> bool {
    self.0.binary_search(&Self::key(i0, i1, fp)).is_ok()
  }

  #[inline]
  fn insert(&mut self, i0: usize, i1: usize, fp: u64) {
    let item = Self::key(i0, i1, fp);
    let idx = self.0.binary_search(&item).unwrap_or_else(|i| i);
    self.0.insert(idx, item);
  }

  #[inline]
  fn remove(&mut self, i0: usize, i1: usize, fp: u64) -> bool {
    if let Ok(idx) = self.0.binary_search(&Self::key(i0, i1, fp)) {
      self.0.remove(idx);
      return true;
    }
    false
  }
}
