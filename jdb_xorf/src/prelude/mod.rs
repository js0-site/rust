//! xor filter 的通称方法
//! Common methods for xor filters.

pub mod bfuse;

use alloc::{boxed::Box, vec::Vec};

use crate::Fingerprint;

/// Helper function for unified Vec allocation in batch queries.
/// 批量查询统一分配 Vec 辅助函数。
#[inline]
pub(crate) fn batch_vec<F>(len: usize, f: F) -> Vec<bool>
where
  F: FnOnce(&mut [bool]),
{
  let mut results = vec![false; len];
  f(&mut results);
  results
}

/// Helper for 4-way chunked batch execution with remainder handling.
/// 4 元素分块批处理辅助函数（包含余数处理）。
#[inline(always)]
pub(crate) fn batch_chunks_4<T, F, R>(
  items: &[T],
  results: &mut [bool],
  mut chunk_fn: F,
  mut rem_fn: R,
) where
  F: FnMut(&[T; 4], &mut [bool; 4]),
  R: FnMut(&T) -> bool,
{
  let (chunks, rem_items) = items.as_chunks::<4>();
  let (res_chunks, rem_res) = results.as_chunks_mut::<4>();
  for (chunk, r_chunk) in chunks.iter().zip(res_chunks.iter_mut()) {
    chunk_fn(chunk, r_chunk);
  }
  for (item, r) in rem_items.iter().zip(rem_res.iter_mut()) {
    *r = rem_fn(item);
  }
}

/// Creates a block to store output fingerprints.
/// 创建一个存储输出指纹的块。
///
/// Under `uniform-random`, uninitialized slots are filled with pseudorandom fingerprints
/// to avoid higher false-positive rates when fingerprint(x) = 0.
/// 在 `uniform-random` 特性下，未使用的槽位填充伪随机指纹，避免 fingerprint(x) = 0 时误报率偏高。
#[inline]
pub(crate) fn make_fp_block<T: Fingerprint>(size: usize) -> Box<[T]> {
  #[cfg(feature = "uniform-random")]
  {
    let mut seed = 0x1234_5678_u64;
    core::iter::repeat_with(|| {
      seed = seed.wrapping_add(1);
      T::from_hash(crate::mix64(seed))
    })
    .take(size)
    .collect::<Box<[_]>>()
  }

  #[cfg(not(feature = "uniform-random"))]
  {
    vec![T::default(); size].into_boxed_slice()
  }
}

/// Checks if a slice of keys has all distinct values.
/// 检查键切片中的所有值是否完全不同。
#[cfg(debug_assertions)]
pub fn all_distinct(keys: &[u64]) -> bool {
  if keys.len() <= 1 {
    return true;
  }

  let mut is_sorted = true;
  for w in keys.windows(2) {
    if w[0] == w[1] {
      return false;
    }
    if w[0] > w[1] {
      is_sorted = false;
      break;
    }
  }

  if is_sorted {
    return true;
  }

  let mut v = keys.to_vec();
  v.sort_unstable();
  v.windows(2).all(|w| w[0] != w[1])
}
