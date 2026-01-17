use libm::round;

use crate::prelude::bfuse::{seg_len, size_factor};

/// Calculates the size information for the filter.
/// 计算过滤器的尺寸信息
pub fn calculate_size(size: usize) -> (u32, u32, u32, u32, u32) {
  let arity = 3u32;
  let seg_len = seg_len(arity, size as u32).min(262144);
  let seg_len_mask = seg_len - 1;
  let size_factor = size_factor(arity, size as u32);
  let capacity = if size > 1 {
    round(size as f64 * size_factor) as u32
  } else {
    0
  };
  let init_seg_count = capacity.div_ceil(seg_len);
  let (fp_array_len, seg_count) = {
    let array_len = init_seg_count * seg_len;
    let seg_count = if array_len.div_ceil(seg_len) < arity {
      1
    } else {
      array_len.div_ceil(seg_len) - (arity - 1)
    };
    ((seg_count + arity - 1) * seg_len, seg_count)
  };
  (
    seg_len,
    seg_len_mask,
    fp_array_len,
    seg_count,
    seg_count * seg_len,
  )
}
