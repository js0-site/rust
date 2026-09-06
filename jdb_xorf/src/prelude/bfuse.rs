//! Binary Fuse 过滤器基础逻辑
//! Core logic for Binary Fuse filters.
// Port of https://github.com/FastFilter/xorfilter/blob/master/binaryfusefilter.go

use libm::{floor, fmax, log};

const LOG_3_33: f64 = 1.2029723223049306;
const INV_LOG_3_33: f64 = 1.0 / LOG_3_33;
const LOG_1000000_DIV_4: f64 = 13.815510557964274 * 0.25;

#[inline(always)]
pub fn seg_len(size: u32) -> u32 {
  if size <= 1 {
    return 4;
  }
  let log_size = log(size as f64);
  1 << (floor(log_size * INV_LOG_3_33 + 2.25) as u32)
}

#[inline(always)]
pub fn size_factor(size: u32) -> f64 {
  if size <= 1 {
    return 1.125;
  }
  fmax(1.125_f64, 0.875 + LOG_1000000_DIV_4 / log(size as f64))
}

#[inline(always)]
pub const fn hash_of_hash(
  hash: u64,
  seg_len: u32,
  seg_len_mask: u32,
  seg_count_len: u32,
) -> (u32, u32, u32) {
  let hi = ((hash as u128 * seg_count_len as u128) >> 64) as u64;
  let h0 = hi as u32;
  let mut h1 = h0 + seg_len;
  let mut h2 = h1 + seg_len;
  h1 ^= ((hash >> 18) as u32) & seg_len_mask;
  h2 ^= (hash as u32) & seg_len_mask;
  (h0, h1, h2)
}

#[inline(always)]
pub const fn mod3(x: u8) -> u8 {
  if x > 2 { x - 3 } else { x }
}

/// Binary Fuse 描述符
/// Binary Fuse desc
#[cfg_attr(feature = "bitcode", derive(bitcode::Decode, bitcode::Encode))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Desc {
  pub seed: u64,
  pub seg_len: u32,
  pub seg_len_mask: u32,
  pub seg_count_len: u32,
}

impl Desc {
  /// Validates the descriptor parameters against fingerprint array length.
  /// 校验描述符参数与指纹数组长度的一致性。
  #[inline]
  pub fn validate(&self, fp_len: usize) -> bool {
    self.seg_len >= 4
      && self.seg_len.is_power_of_two()
      && self.seg_len_mask == self.seg_len - 1
      && self
        .seg_count_len
        .checked_add(self.seg_len * 2)
        .is_some_and(|min_len| fp_len >= min_len as usize)
  }
}

impl Default for Desc {
  #[inline]
  fn default() -> Self {
    Self {
      seed: 0,
      seg_len: 4,
      seg_len_mask: 3,
      seg_count_len: 4,
    }
  }
}
