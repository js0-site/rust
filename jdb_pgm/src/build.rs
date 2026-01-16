//! Segment building using Optimal Piecewise Linear Approximation (Shrinking Cone)
//! 使用最优分段线性逼近（收缩锥算法）构建段
//!
//! Time Complexity: O(N)

#![allow(clippy::cast_precision_loss)]

use crate::{
  Key, Segment,
  consts::{LUT_BINS_MULTIPLIER, MAX_LUT_BINS, MIN_LUT_BINS},
};

/// Build segments using the streaming shrinking cone algorithm.
/// 使用流式收缩锥算法构建段 (O(N))
pub fn build_segments<K: Key>(data: &[K], epsilon: usize) -> Vec<Segment<K>> {
  let n = data.len();
  if n == 0 {
    return vec![];
  }

  let estimated_segments = (n / (epsilon * 2).max(1)).clamp(16, 1 << 20);
  let mut segments = Vec::with_capacity(estimated_segments);

  let mut start = 0;
  let eps = epsilon as f64;
  let ptr = data.as_ptr();

  while start < n {
    let first_key = unsafe { (*ptr.add(start)).as_f64() };
    let first_idx = start as f64;

    let mut min_slope = f64::NEG_INFINITY;
    let mut max_slope = f64::INFINITY;

    let mut end = start + 1;

    while end < n {
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
      min_key: unsafe { *data.get_unchecked(start) },
      max_key: unsafe { *data.get_unchecked(end - 1) },
      slope,
      intercept,
      start_idx: start as u32,
      end_idx: end as u32,
    });

    start = end;
  }

  segments
}

/// Build lookup table for fast segment search
/// 构建查找表以快速搜索段
pub fn build_lut<K: Key>(data: &[K], segments: &[Segment<K>]) -> (Vec<u32>, f64, f64) {
  if data.is_empty() || segments.is_empty() {
    return (vec![0], 0.0, 0.0);
  }

  let bins = (segments.len() * LUT_BINS_MULTIPLIER).clamp(MIN_LUT_BINS, MAX_LUT_BINS);

  let min_key = unsafe { data.get_unchecked(0) }.as_f64();
  let max_key = unsafe { data.get_unchecked(data.len() - 1) }.as_f64();

  let span = (max_key - min_key).max(1.0);
  let scale = bins as f64 / span;

  let mut lut = vec![0u32; bins + 1];
  let mut seg_idx = 0u32;
  let seg_len = segments.len();

  for (b, slot) in lut.iter_mut().enumerate() {
    let key_at_bin = min_key + (b as f64) / scale;

    while (seg_idx as usize) + 1 < seg_len {
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
