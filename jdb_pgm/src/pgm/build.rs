//! Segment building using Optimal Piecewise Linear Approximation (Shrinking Cone)
//! 使用最优分段线性逼近（收缩锥算法）构建段
//!
//! Time Complexity: O(N)

#![allow(clippy::cast_precision_loss)]

use super::{
  consts::{LUT_BINS_MULTIPLIER, MAX_LUT_BINS, MIN_LUT_BINS},
  types::{Key, Segment},
};

/// Build segments using the streaming shrinking cone algorithm.
/// 使用流式收缩锥算法构建段 (O(N))
pub fn build_segments<K: Key>(sorted: &[K], epsilon: usize) -> Vec<Segment<K>> {
  let n = sorted.len();
  if n == 0 {
    return vec![];
  }

  let estimated_segments = (n / (epsilon * 2).max(1)).max(16);
  let mut segments = Vec::with_capacity(estimated_segments);

  let mut start = 0;
  let eps = epsilon as f64;
  let ptr = sorted.as_ptr();

  while start < n {
    // SAFETY: start is less than n, so ptr.add(start) is within bounds.
    let first_key = unsafe { (*ptr.add(start)).as_f64() };
    let first_idx = start as f64;

    let mut min_slope = f64::NEG_INFINITY;
    let mut max_slope = f64::INFINITY;

    let mut end = start + 1;

    while end < n {
      // SAFETY: end is checked < n, so ptr.add(end) is valid.
      let key = unsafe { (*ptr.add(end)).as_f64() };
      let idx = end as f64;
      let dx = key - first_key;

      if dx == 0.0 {
        if (idx - first_idx) > (2 * epsilon) as f64 {
          break;
        }
        end += 1;
        continue;
      }

      let slope_lo = (idx - first_idx - eps) / dx;
      let slope_hi = (idx - first_idx + eps) / dx;

      let new_min = min_slope.max(slope_lo);
      let new_max = max_slope.min(slope_hi);

      if new_min > new_max {
        break;
      }

      min_slope = new_min;
      max_slope = new_max;
      end += 1;
    }

    let slope = if end == start + 1 {
      0.0
    } else {
      (min_slope + max_slope) * 0.5
    };

    let intercept = first_idx - slope * first_key;

    segments.push(Segment {
      // SAFETY: indices are within bounds [0, n).
      min_key: unsafe { *sorted.get_unchecked(start) },
      max_key: unsafe { *sorted.get_unchecked(end - 1) },
      slope,
      intercept,
      start_idx: start,
      end_idx: end,
    });

    start = end;
  }

  segments
}

/// Build lookup table for fast segment search
/// 构建查找表以快速搜索段
pub fn build_lut<K: Key>(sorted: &[K], segments: &[Segment<K>]) -> (Vec<u32>, f64, f64) {
  if sorted.is_empty() || segments.is_empty() {
    return (vec![0], 0.0, 0.0);
  }

  let bins = (segments.len() * LUT_BINS_MULTIPLIER).clamp(MIN_LUT_BINS, MAX_LUT_BINS);

  // SAFETY: checked emptiness above.
  let min_key = unsafe { sorted.get_unchecked(0) }.as_f64();
  let max_key = unsafe { sorted.get_unchecked(sorted.len() - 1) }.as_f64();

  let span = (max_key - min_key).max(1.0);
  let scale = bins as f64 / span;

  let mut lut = vec![0u32; bins + 1];
  let mut seg_idx = 0u32;
  let seg_len = segments.len();

  for (b, slot) in lut.iter_mut().enumerate() {
    let key_at_bin = min_key + (b as f64) / scale;

    while (seg_idx as usize) + 1 < seg_len {
      // SAFETY: seg_idx + 1 < seg_len, so seg_idx is valid.
      let seg_max = unsafe { segments.get_unchecked(seg_idx as usize).max_key }.as_f64();
      if seg_max >= key_at_bin {
        break;
      }
      seg_idx += 1;
    }
    *slot = seg_idx;
  }

  (lut, scale, min_key)
}
