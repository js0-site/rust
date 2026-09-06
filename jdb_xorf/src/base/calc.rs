//! 过滤器尺寸与段分配计算
//! Filter size and segment calculation.

use libm::round;

use crate::prelude::bfuse::{seg_len, size_factor};

/// Binary Fuse 过滤器的超图超边度数（3 元超图）
pub const ARITY: u32 = 3;

/// 单段最大容量限制（2^18）
pub const MAX_SEG_LEN: u32 = 262_144;

/// 过滤器尺寸与段配置信息
/// Filter size and segment configuration information.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SizeInfo {
  /// 段长度（保证为 2 的幂）
  pub seg_len: u32,
  /// 段长度掩码（seg_len - 1）
  pub seg_len_mask: u32,
  /// 指纹数组总长度
  pub fp_array_len: u32,
  /// 独立段数量
  pub seg_count: u32,
  /// 有效段总跨度（seg_count * seg_len）
  pub seg_count_len: u32,
}

impl SizeInfo {
  /// 计算给定键数量的过滤器尺寸信息
  #[inline]
  pub fn from_size(size: usize) -> Self {
    let seg_len = seg_len(size as u32).min(MAX_SEG_LEN);
    let seg_len_mask = seg_len - 1;
    let size_factor = size_factor(size as u32);
    let capacity = if size > 1 {
      round(size as f64 * size_factor) as u32
    } else {
      0
    };
    let init_seg_count = capacity.div_ceil(seg_len);
    let seg_count = if init_seg_count < ARITY {
      1
    } else {
      init_seg_count - (ARITY - 1)
    };
    let fp_array_len = (seg_count + ARITY - 1) * seg_len;
    Self {
      seg_len,
      seg_len_mask,
      fp_array_len,
      seg_count,
      seg_count_len: seg_count * seg_len,
    }
  }
}
