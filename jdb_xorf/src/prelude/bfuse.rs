//! Binary Fuse 过滤器基础逻辑
//! Core logic for Binary Fuse filters.
// Port of https://github.com/FastFilter/xorfilter/blob/master/binaryfusefilter.go

use libm::{floor, fmax, log};

#[inline(always)]
pub fn seg_len(arity: u32, size: u32) -> u32 {
  if size == 0 {
    return 4;
  }
  match arity {
    3 => 1 << (floor(log(size as f64) / log(3.33_f64) + 2.25) as u32),
    4 => 1 << (floor(log(size as f64) / log(2.91_f64) - 0.5) as u32),
    _ => 65536,
  }
}

#[inline(always)]
pub fn size_factor(arity: u32, size: u32) -> f64 {
  match arity {
    3 => fmax(
      1.125_f64,
      0.875 + 0.25 * log(1000000_f64) / log(size as f64),
    ),
    4 => fmax(1.075_f64, 0.77 + 0.305 * log(600000_f64) / log(size as f64)),
    _ => 2.0,
  }
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Desc {
  pub seed: u64,
  pub seg_len: u32,
  pub seg_len_mask: u32,
  pub seg_count_len: u32,
}
