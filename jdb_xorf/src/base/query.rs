//! 查询路径的核心底层实现
//! Core query path low-level implementation.

use crate::{Fingerprint, hash_of_hash, mix_key};

/// Branchless core hash query (strictly 3 memory reads, zero branch misprediction penalty).
/// 核心无分支哈希查询（严格 3 次内存读取，零分支惩罚）。
///
/// # Safety
/// Caller must ensure `fingerprints` has at least `seg_count_len + 2 * seg_len` elements,
/// and `seg_len` is a power of two with `seg_len_mask = seg_len - 1`.
/// 调用者必须确保 `fingerprints` 长度至少为 `seg_count_len + 2 * seg_len`，
/// 且 `seg_len` 为 2 的幂并满足 `seg_len_mask = seg_len - 1`。
#[inline(always)]
pub unsafe fn query_hash_unchecked<T: Fingerprint>(
  hash: u64,
  fingerprints: &[T],
  seg_len: u32,
  seg_len_mask: u32,
  seg_count_len: u32,
) -> bool {
  let f = T::from_hash(hash);
  let (h0, h1, h2) = hash_of_hash(hash, seg_len, seg_len_mask, seg_count_len);
  let fp = unsafe {
    *fingerprints.get_unchecked(h0 as usize)
      ^ *fingerprints.get_unchecked(h1 as usize)
      ^ *fingerprints.get_unchecked(h2 as usize)
  };
  f == fp
}

/// 4-way instruction-level parallelism (ILP) hash probing with interleaved loads.
/// 4 路指令级并行（ILP）哈希探测实现（严格 12 次交错内存读取，隐藏访存延迟）。
///
/// # Safety
/// Same preconditions as [`query_hash_unchecked`].
/// 安全前置条件同 [`query_hash_unchecked`]。
#[inline(always)]
pub unsafe fn query_4_hashes_unchecked<T: Fingerprint>(
  hashes: [u64; 4],
  fingerprints: &[T],
  seg_len: u32,
  seg_len_mask: u32,
  seg_count_len: u32,
) -> [bool; 4] {
  let [hash0, hash1, hash2, hash3] = hashes;

  let f0 = T::from_hash(hash0);
  let f1 = T::from_hash(hash1);
  let f2 = T::from_hash(hash2);
  let f3 = T::from_hash(hash3);

  let (h0_0, h0_1, h0_2) = hash_of_hash(hash0, seg_len, seg_len_mask, seg_count_len);
  let (h1_0, h1_1, h1_2) = hash_of_hash(hash1, seg_len, seg_len_mask, seg_count_len);
  let (h2_0, h2_1, h2_2) = hash_of_hash(hash2, seg_len, seg_len_mask, seg_count_len);
  let (h3_0, h3_1, h3_2) = hash_of_hash(hash3, seg_len, seg_len_mask, seg_count_len);

  unsafe {
    // Interleave loads across all 4 keys to maximize LFB (Line Fill Buffer) concurrency
    // 在 4 个键之间交错读取，以最大化利用 CPU 缓冲行填充缓冲区（LFB）并发能力
    let p0_0 = *fingerprints.get_unchecked(h0_0 as usize);
    let p1_0 = *fingerprints.get_unchecked(h1_0 as usize);
    let p2_0 = *fingerprints.get_unchecked(h2_0 as usize);
    let p3_0 = *fingerprints.get_unchecked(h3_0 as usize);

    let p0_1 = *fingerprints.get_unchecked(h0_1 as usize);
    let p1_1 = *fingerprints.get_unchecked(h1_1 as usize);
    let p2_1 = *fingerprints.get_unchecked(h2_1 as usize);
    let p3_1 = *fingerprints.get_unchecked(h3_1 as usize);

    let p0_2 = *fingerprints.get_unchecked(h0_2 as usize);
    let p1_2 = *fingerprints.get_unchecked(h1_2 as usize);
    let p2_2 = *fingerprints.get_unchecked(h2_2 as usize);
    let p3_2 = *fingerprints.get_unchecked(h3_2 as usize);

    let fp0 = p0_0 ^ p0_1 ^ p0_2;
    let fp1 = p1_0 ^ p1_1 ^ p1_2;
    let fp2 = p2_0 ^ p2_1 ^ p2_2;
    let fp3 = p3_0 ^ p3_1 ^ p3_2;

    [f0 == fp0, f1 == fp1, f2 == fp2, f3 == fp3]
  }
}

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
  #[cfg(debug_assertions)]
  debug_assert!(
    fingerprints.len() >= (seg_count_len + 2 * seg_len) as usize,
    "fingerprints length mismatch with descriptor"
  );
  let hash = mix_key(key, seed);
  unsafe { query_hash_unchecked(hash, fingerprints, seg_len, seg_len_mask, seg_count_len) }
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
  #[cfg(debug_assertions)]
  debug_assert!(
    fingerprints.len() >= (seg_count_len + 2 * seg_len) as usize,
    "fingerprints length mismatch with descriptor"
  );

  crate::batch_chunks_4(
    keys,
    results,
    |k_chunk, r_chunk| {
      let hashes = [
        mix_key(k_chunk[0], seed),
        mix_key(k_chunk[1], seed),
        mix_key(k_chunk[2], seed),
        mix_key(k_chunk[3], seed),
      ];
      *r_chunk = unsafe {
        query_4_hashes_unchecked(hashes, fingerprints, seg_len, seg_len_mask, seg_count_len)
      };
    },
    |&k| {
      let hash = mix_key(k, seed);
      unsafe { query_hash_unchecked(hash, fingerprints, seg_len, seg_len_mask, seg_count_len) }
    },
  );
}
