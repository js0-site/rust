//! 过滤器尺寸与段分配计算
//! Filter size and segment calculation.

use libm::round;

use crate::{seg_len, size_factor};

/// Degree of hyperedges in Binary Fuse filter (3-uniform hypergraph).
/// Binary Fuse 过滤器的超图超边度数（3 元超图）。
pub const ARITY: u32 = 3;

/// Maximum segment capacity limit (2^18).
/// 单段最大容量限制（2^18）。
pub const MAX_SEG_LEN: u32 = 262_144;

/// Maximum supported key count for 32-bit indexing (~1 billion).
/// 32 位索引支持的最大键容量上限（约 10 亿）。
pub const MAX_CAPACITY: usize = 1_000_000_000;

/// Filter size and segment configuration information.
/// 过滤器尺寸与段配置信息。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SizeInfo {
  /// Segment length (guaranteed to be a power of two).
  /// 段长度（保证为 2 的幂）。
  pub seg_len: u32,
  /// Segment length mask (seg_len - 1).
  /// 段长度掩码（seg_len - 1）。
  pub seg_len_mask: u32,
  /// Total length of the fingerprint array.
  /// 指纹数组总长度。
  pub fp_array_len: u32,
  /// Number of independent segments.
  /// 独立段数量。
  pub seg_count: u32,
  /// Total span of valid segments (seg_count * seg_len).
  /// 有效段总跨度（seg_count * seg_len）。
  pub seg_count_len: u32,
}

impl SizeInfo {
  /// Computes filter size information for a given key count.
  /// 计算给定键数量的过滤器尺寸信息。
  #[inline]
  pub fn from_size(size: usize) -> Self {
    let size = size.min(MAX_CAPACITY);
    let seg_len = seg_len(size as u32).min(MAX_SEG_LEN);
    let seg_len_mask = seg_len - 1;
    let capacity = if size > 1 {
      let size_factor = size_factor(size as u32);
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
