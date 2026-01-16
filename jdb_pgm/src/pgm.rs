//! Pgm-Index core (no data ownership)
//! Pgm 索引核心（不持有数据）

#![allow(clippy::cast_precision_loss)]

use std::mem::size_of;

use crate::{
  Key, PgmError, Result, Segment,
  build::{build_lut, build_segments},
  consts::MIN_EPSILON,
};

/// Pgm-Index core structure (no data ownership, serializable)
/// Pgm 索引核心结构（不持有数据，可序列化）
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
#[derive(Clone, Debug)]
pub struct Pgm<K: Key> {
  pub epsilon: usize,
  pub segments: Vec<Segment<K>>,
  pub lut: Vec<u32>,
  pub scale: f64,
  pub min_key: f64,
  pub len: usize,
}

impl<K: Key> Pgm<K> {
  /// Build Pgm from sorted data slice (O(N) build time)
  /// 从已排序数据切片构建 Pgm
  ///
  /// # Errors
  /// Returns `PgmError::InvalidEpsilon` if epsilon < `MIN_EPSILON`
  /// Returns `PgmError::EmptyData` if data is empty
  /// Returns `PgmError::NotSorted` if `check_sorted` is true and data is not sorted
  pub fn new(data: &[K], epsilon: usize, check_sorted: bool) -> Result<Self> {
    if epsilon < MIN_EPSILON {
      return Err(PgmError::InvalidEpsilon {
        provided: epsilon,
        min: MIN_EPSILON,
      });
    }
    if data.is_empty() {
      return Err(PgmError::EmptyData);
    }

    if check_sorted && !is_sorted(data) {
      return Err(PgmError::NotSorted);
    }

    let segments = build_segments(data, epsilon);
    let (lut, scale, min_key) = build_lut(data, &segments);

    Ok(Self {
      epsilon,
      segments,
      lut,
      scale,
      min_key,
      len: data.len(),
    })
  }

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
    self.segments.len() * size_of::<Segment<K>>() + self.lut.len() * size_of::<u32>()
  }

  /// Predict position for a key
  /// 预测键的位置
  #[inline]
  #[must_use]
  pub fn predict(&self, key: K) -> usize {
    let seg = self.find_seg(key);
    predict_in_seg(seg, key.as_f64())
  }

  /// Predict search range [start, end) for a key
  /// 预测键的搜索范围 [start, end)
  #[inline]
  #[must_use]
  pub fn predict_range(&self, key: K) -> (usize, usize) {
    let seg = self.find_seg(key);
    let pred = predict_in_seg(seg, key.as_f64());
    let start = pred
      .saturating_sub(self.epsilon)
      .max(seg.start_idx as usize);
    let end = (pred + self.epsilon + 1).min(seg.end_idx as usize);
    (start, end)
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
    let (lo, hi) = self.predict_range(k);
    let key_bytes = key.as_bytes();
    // Binary search in predicted range using bytes comparison
    // 在预测范围内使用字节比较二分查找
    let mut left = lo;
    let mut right = hi;
    while left < right {
      let mid = left + (right - left) / 2;
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
    let (lo, hi) = self.predict_range(key);
    let mut left = lo;
    let mut right = hi;
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

      let mut idx = unsafe { *self.lut.get_unchecked(bin) } as usize;
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
}

/// Predict index position using segment's linear model
/// 使用段的线性模型预测索引位置
#[inline]
fn predict_in_seg(seg: &Segment<impl Key>, key_f64: f64) -> usize {
  let pos = seg.slope.mul_add(key_f64, seg.intercept) + 0.5;
  let lo = seg.start_idx as usize;
  let hi = (seg.end_idx - 1) as usize;
  (pos as usize).clamp(lo, hi)
}

/// Check if data is sorted (early termination)
/// 检查数据是否已排序（提前终止）
#[inline]
fn is_sorted<K: Ord>(data: &[K]) -> bool {
  data.windows(2).all(|w| w[0] <= w[1])
}
