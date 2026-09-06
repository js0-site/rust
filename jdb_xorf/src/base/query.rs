//! 查询路径的核心底层实现
//! Core query path low-level implementation.

use crate::{
  hash::{Fingerprint, mix_key},
  prelude::bfuse::hash_of_hash,
};

/// 检查单个键是否存在
/// Checks if a key exists in the filter.
#[inline(always)]
pub fn contains_impl<T: Fingerprint>(
  key: u64,
  seed: u64,
  fingerprints: &[T],
  seg_len: u32,
  seg_len_mask: u32,
  seg_count_len: u32,
) -> bool {
  if fingerprints.is_empty() {
    return false;
  }
  let hash = mix_key(key, seed);
  let f = T::from_hash(hash);
  let (h0, h1, h2) = hash_of_hash(hash, seg_len, seg_len_mask, seg_count_len);
  unsafe {
    let fp = *fingerprints.get_unchecked(h0 as usize)
      ^ *fingerprints.get_unchecked(h1 as usize)
      ^ *fingerprints.get_unchecked(h2 as usize);
    f == fp
  }
}

/// 4 路指令级并行（ILP）批量查询实现
/// 4-way instruction-level parallelism batch query implementation.
#[inline]
pub fn contains_batch_impl<T: Fingerprint>(
  keys: &[u64],
  results: &mut [bool],
  seed: u64,
  fingerprints: &[T],
  seg_len: u32,
  seg_len_mask: u32,
  seg_count_len: u32,
) {
  debug_assert_eq!(keys.len(), results.len());
  if fingerprints.is_empty() {
    results.fill(false);
    return;
  }

  let (chunks, remainder_keys) = keys.as_chunks::<4>();
  let (res_chunks, remainder_results) = results.as_chunks_mut::<4>();

  for (k_chunk, r_chunk) in chunks.iter().zip(res_chunks.iter_mut()) {
    let k0 = k_chunk[0];
    let k1 = k_chunk[1];
    let k2 = k_chunk[2];
    let k3 = k_chunk[3];

    let hash0 = mix_key(k0, seed);
    let hash1 = mix_key(k1, seed);
    let hash2 = mix_key(k2, seed);
    let hash3 = mix_key(k3, seed);

    let f0 = T::from_hash(hash0);
    let f1 = T::from_hash(hash1);
    let f2 = T::from_hash(hash2);
    let f3 = T::from_hash(hash3);

    let (h0_0, h0_1, h0_2) = hash_of_hash(hash0, seg_len, seg_len_mask, seg_count_len);
    let (h1_0, h1_1, h1_2) = hash_of_hash(hash1, seg_len, seg_len_mask, seg_count_len);
    let (h2_0, h2_1, h2_2) = hash_of_hash(hash2, seg_len, seg_len_mask, seg_count_len);
    let (h3_0, h3_1, h3_2) = hash_of_hash(hash3, seg_len, seg_len_mask, seg_count_len);

    unsafe {
      let fp0 = *fingerprints.get_unchecked(h0_0 as usize)
        ^ *fingerprints.get_unchecked(h0_1 as usize)
        ^ *fingerprints.get_unchecked(h0_2 as usize);
      let fp1 = *fingerprints.get_unchecked(h1_0 as usize)
        ^ *fingerprints.get_unchecked(h1_1 as usize)
        ^ *fingerprints.get_unchecked(h1_2 as usize);
      let fp2 = *fingerprints.get_unchecked(h2_0 as usize)
        ^ *fingerprints.get_unchecked(h2_1 as usize)
        ^ *fingerprints.get_unchecked(h2_2 as usize);
      let fp3 = *fingerprints.get_unchecked(h3_0 as usize)
        ^ *fingerprints.get_unchecked(h3_1 as usize)
        ^ *fingerprints.get_unchecked(h3_2 as usize);

      r_chunk[0] = f0 == fp0;
      r_chunk[1] = f1 == fp1;
      r_chunk[2] = f2 == fp2;
      r_chunk[3] = f3 == fp3;
    }
  }

  for (k, r) in remainder_keys.iter().zip(remainder_results) {
    *r = contains_impl(*k, seed, fingerprints, seg_len, seg_len_mask, seg_count_len);
  }
}
