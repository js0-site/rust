//! Pgm-Index core (no data ownership)
//! Pgm 索引核心（不持有数据）

#![allow(clippy::cast_precision_loss)]

use std::{mem::size_of, ops::Range};

use crate::{
  Key, Segment,
  build::{build_lut, build_segments},
  consts::MIN_EPSILON,
  patch::{NoPatch, Patch},
};

/// Pgm-Index core structure (no data ownership, serializable)
/// Pgm 索引核心结构（不持有数据，可序列化）
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
#[derive(Clone, Debug)]
pub struct Pgm<K: Key, P = NoPatch> {
  pub epsilon: usize,
  pub segments: Vec<Segment<K>>,
  pub lut: Vec<u32>,
  pub scale: f64,
  pub min_key: f64,
  pub len: usize,
  /// The patch data (Zero-size for NoPatch)
  pub patch: P,
}

impl<K: Key> Pgm<K, NoPatch> {
  /// Build Pgm from sorted data slice (O(N) build time)
  /// 从已排序数据切片构建 Pgm
  pub fn new(sorted: &[K], epsilon: usize) -> Self {
    let epsilon = epsilon.max(MIN_EPSILON);
    let len = sorted.len();
    if len == 0 {
      return Self {
        epsilon,
        segments: vec![],
        lut: vec![0],
        scale: 0.0,
        min_key: 0.0,
        len: 0,
        patch: NoPatch,
      };
    }

    let segments = build_segments(sorted, epsilon);
    let (lut, scale, min_key) = build_lut(sorted, &segments);

    Self {
      epsilon,
      segments,
      lut,
      scale,
      min_key,
      len,
      patch: NoPatch,
    }
  }
}

impl<K: Key, P: Patch<K>> Pgm<K, P> {
  /// Data length
  /// 数据长度
  #[inline]
  #[must_use]
  pub fn len(&self) -> usize {
    self.len
  }

  #[inline]
  #[must_use]
  pub fn is_empty(&self) -> bool {
    self.len == 0
  }

  #[inline]
  #[must_use]
  pub fn segment_count(&self) -> usize {
    self.segments.len()
  }

  #[inline]
  #[must_use]
  pub fn avg_segment_size(&self) -> f64 {
    self.len as f64 / self.segments.len().max(1) as f64
  }

  /// Memory usage (excluding data)
  /// 内存占用（不含数据）
  #[inline]
  #[must_use]
  pub fn mem_usage(&self) -> usize {
    self.segments.len() * size_of::<Segment<K>>()
      + self.lut.len() * size_of::<u32>()
      + size_of::<P>()
  }

  /// Predict position for a key
  /// 预测键的位置
  #[inline]
  #[must_use]
  pub fn predict(&self, key: K) -> usize {
    if self.segments.is_empty() {
      return 0;
    }
    let seg = self.find_seg(key);
    predict_in_seg(seg, key.as_f64())
  }

  /// Predict search range [start, end) for a key
  /// 预测键的搜索范围 [start, end)
  ///
  /// Returns a standard Rust `Range<usize>` where:
  /// - `range.start` is inclusive (contains the key)
  /// - `range.end` is exclusive (max is `self.len()`)
  ///
  /// 返回标准 Rust Range，其中：
  /// - `range.start` 包含（可能包含键）
  /// - `range.end` 不包含（最大为 `self.len()`）
  #[inline]
  #[must_use]
  pub fn predict_range(&self, key: K) -> Range<usize> {
    if self.segments.is_empty() {
      return 0..0;
    }
    let seg = self.find_seg(key);
    let pred = predict_in_seg(seg, key.as_f64());
    let start = pred.saturating_sub(self.epsilon).max(seg.start_idx);
    let end = (pred + self.epsilon + 1).min(seg.end_idx);
    start..end
  }

  /// Find index using PGM prediction + binary search (bytes comparison)
  /// 使用 PGM 预测 + 二分查找定位索引（字节比较）
  ///
  /// `get_key`: closure to get key bytes at index
  /// `get_key`：获取索引处键字节的闭包
  ///
  /// Returns index where key would be inserted (like partition_point)
  /// 返回键应插入的位置（类似 partition_point）
  #[inline]
  pub fn find<'a, Q, F>(&self, key: &Q, get_key: F) -> usize
  where
    Q: crate::ToKey<K> + ?Sized,
    F: Fn(usize) -> Option<&'a [u8]>,
  {
    let k = key.to_key();
    let range = self.predict_range(k);
    let key_bytes = key.as_bytes();
    // Binary search in predicted range using bytes comparison
    // 在预测范围内使用字节比较二分查找
    let mut left = range.start;
    let mut right = range.end;
    while left < right {
      let mid = left + (right - left) / 2;
      // SAFETY: mid is always within [0, self.len()) because range is bounded by len.
      // Callers of find must ensure get_key handles mid correctly or is safe.
      match get_key(mid) {
        Some(mk) if mk < key_bytes => left = mid + 1,
        _ => right = mid,
      }
    }
    left
  }

  /// Find index using PGM prediction + binary search (Key type comparison)
  /// 使用 PGM 预测 + 二分查找定位索引（Key 类型比较）
  #[inline]
  pub fn find_key<F>(&self, key: K, get_key: F) -> usize
  where
    F: Fn(usize) -> Option<K>,
  {
    let range = self.predict_range(key);
    let mut left = range.start;
    let mut right = range.end;
    while left < right {
      let mid = left + (right - left) / 2;
      match get_key(mid) {
        Some(k) if k < key => left = mid + 1,
        _ => right = mid,
      }
    }
    left
  }

  /// Find segment for a key
  /// 查找键所属的段
  #[inline]
  fn find_seg(&self, key: K) -> &Segment<K> {
    // SAFETY: This function is only called when self.segments is not empty.
    // Checked in predict() and predict_range().
    if self.segments.len() <= 1 {
      unsafe { self.segments.get_unchecked(0) }
    } else {
      let y = key.as_f64();
      let idx_candidate = (y - self.min_key) * self.scale;
      let lut_max = (self.lut.len() - 1) as isize;

      let idx_i = idx_candidate as isize;
      let bin = if idx_i < 0 {
        0
      } else if idx_i >= lut_max {
        lut_max as usize
      } else {
        idx_i as usize
      };

      // SAFETY: bin is clamped to [0, lut.len()-1]
      let mut idx = unsafe { *self.lut.get_unchecked(bin) } as usize;

      // SAFETY: idx from lut is a valid segment index.
      // We check bounds in the loops.
      let mut seg = unsafe { self.segments.get_unchecked(idx) };

      while idx + 1 < self.segments.len() {
        if key <= seg.max_key {
          break;
        }
        idx += 1;
        seg = unsafe { self.segments.get_unchecked(idx) };
      }

      while idx > 0 {
        if key >= seg.min_key {
          break;
        }
        idx -= 1;
        seg = unsafe { self.segments.get_unchecked(idx) };
      }
      seg
    }
  }

  /// Find segment containing the given index
  /// 查找包含给定索引的段
  #[inline]
  pub fn find_segment_for_index(&self, index: usize) -> Option<&Segment<K>> {
    // Segments are sorted by start_idx.
    // We can binary search.
    if self.segments.is_empty() {
      return None;
    }

    let idx = self.segments.partition_point(|seg| seg.start_idx <= index);
    // partition_point returns the first index where predicate is false.
    // So the segment we want is at idx - 1.
    if idx == 0 {
      return None; // Should not happen if segments cover 0..len and index < len
    }
    let seg = &self.segments[idx - 1];
    if index < seg.end_idx { Some(seg) } else { None }
  }
}

/// Predict index position using segment's linear model
/// 使用段的线性模型预测索引位置
#[inline]
fn predict_in_seg(seg: &Segment<impl Key>, key_f64: f64) -> usize {
  let pos = seg.slope.mul_add(key_f64, seg.intercept) + 0.5;
  let lo = seg.start_idx;
  let hi = seg.end_idx - 1;
  (pos as usize).clamp(lo, hi)
}
