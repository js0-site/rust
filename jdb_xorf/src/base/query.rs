use crate::{
  hash::{Fingerprint, mix64},
  prelude::bfuse::hash_of_hash,
};

/// 检查键是否存在
/// Checks if a key exists in the filter.
#[inline(always)]
pub fn contains_impl<T: Fingerprint>(
  key: &u64,
  seed: u64,
  fingerprints: &[T],
  seg_len: u32,
  seg_len_mask: u32,
  seg_count_len: u32,
) -> bool {
  let hash = mix64(key.wrapping_add(seed));
  let f = T::from_hash(hash);
  let (h0, h1, h2) = hash_of_hash(hash, seg_len, seg_len_mask, seg_count_len);
  unsafe {
    let fp = *fingerprints.get_unchecked(h0 as usize)
      ^ *fingerprints.get_unchecked(h1 as usize)
      ^ *fingerprints.get_unchecked(h2 as usize);
    f ^ fp == T::ZERO
  }
}
