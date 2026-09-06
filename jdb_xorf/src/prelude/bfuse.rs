//! Binary Fuse 过滤器基础逻辑
//! Core logic for Binary Fuse filters.
// Port of https://github.com/FastFilter/xorfilter/blob/master/binaryfusefilter.go

use libm::{floor, fmax, log};

const LOG_3_33: f64 = 1.2029723223049306;
const LOG_2_91: f64 = 1.068153096057863;
const LOG_1000000: f64 = 13.815510557964274;
const LOG_600000: f64 = 13.304684931102917;

#[inline(always)]
pub fn seg_len(arity: u32, size: u32) -> u32 {
  if size == 0 {
    return 4;
  }
  let log_size = log(size as f64);
  match arity {
    3 => 1 << (floor(log_size / LOG_3_33 + 2.25) as u32),
    4 => 1 << (floor(log_size / LOG_2_91 - 0.5) as u32),
    _ => 65536,
  }
}

#[inline(always)]
pub fn size_factor(arity: u32, size: u32) -> f64 {
  let log_size = log(size as f64);
  match arity {
    3 => fmax(1.125_f64, 0.875 + 0.25 * LOG_1000000 / log_size),
    4 => fmax(1.075_f64, 0.77 + 0.305 * LOG_600000 / log_size),
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
