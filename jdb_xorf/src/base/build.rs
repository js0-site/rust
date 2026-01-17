use alloc::boxed::Box;

use super::calc::calculate_size;
use crate::{
  hash::{Fingerprint, mix64},
  prelude::bfuse::{Desc, hash_of_hash, mod3},
};

/// 核心构造算法
/// Core construction algorithm.
pub fn make<T: Fingerprint, I>(keys: I, max_iter: usize) -> (Desc, Box<[T]>)
where
  I: ExactSizeIterator<Item = u64> + Clone,
{
  #[cfg(debug_assertions)]
  {
    use crate::prelude::all_distinct;
    assert!(
      all_distinct(keys.clone()),
      "Binary Fuse filters must be constructed from a collection containing all distinct keys."
    );
  }

  let size = keys.len();
  let (seg_len, seg_len_mask, fp_array_len, seg_count, seg_count_len) = calculate_size(size);

  // Initialize fingerprint array
  // 初始化指纹数组
  let mut fingerprints: Box<[T]> = make_fp_block!(fp_array_len, T);

  let mut alone: Box<[u32]> = vec![0; fp_array_len as usize].into_boxed_slice();
  let mut t2count: Box<[u8]> = vec![0; fp_array_len as usize].into_boxed_slice();
  let mut t2hash: Box<[u64]> = vec![0; fp_array_len as usize].into_boxed_slice();
  let mut reverse_h: Box<[u8]> = vec![0; size].into_boxed_slice();
  let mut reverse_order: Box<[u64]> = vec![0; size + 1].into_boxed_slice();
  reverse_order[size] = 1;

  let block_bits = {
    let mut bits = 1;
    while (1 << bits) < seg_count {
      bits += 1;
    }
    bits
  };

  let start_pos_len = 1 << block_bits;
  let mut start_pos: Box<[usize]> = vec![0; start_pos_len].into_boxed_slice();
  let mut seed: u64 = 1;

  for _ in 0..max_iter {
    let current_seed = seed;
    seed = seed.wrapping_add(1);
    for (i, p) in start_pos.iter_mut().enumerate() {
      *p = (((i as u64) * (size as u64)) >> block_bits) as usize;
    }

    for key in keys.clone() {
      let hash = mix64(key.wrapping_add(current_seed));
      let mut seg_index = hash >> (64 - block_bits);
      unsafe {
        while *reverse_order.get_unchecked(*start_pos.get_unchecked(seg_index as usize)) != 0 {
          seg_index += 1;
          seg_index &= (1 << block_bits) - 1;
        }
        *reverse_order.get_unchecked_mut(*start_pos.get_unchecked(seg_index as usize)) = hash;
        *start_pos.get_unchecked_mut(seg_index as usize) += 1;
      }
    }

    let mut error = false;
    let mut duplicates = 0;
    for &hash in reverse_order.iter().take(size) {
      let (i0, i1, i2) = hash_of_hash(hash, seg_len, seg_len_mask, seg_count_len);
      unsafe {
        let (i0, i1, i2) = (i0 as usize, i1 as usize, i2 as usize);
        *t2count.get_unchecked_mut(i0) += 4;
        *t2hash.get_unchecked_mut(i0) ^= hash;
        *t2count.get_unchecked_mut(i1) += 4;
        *t2count.get_unchecked_mut(i1) ^= 1;
        *t2hash.get_unchecked_mut(i1) ^= hash;
        *t2count.get_unchecked_mut(i2) += 4;
        *t2count.get_unchecked_mut(i2) ^= 2;
        *t2hash.get_unchecked_mut(i2) ^= hash;

        if *t2hash.get_unchecked(i0) & *t2hash.get_unchecked(i1) & *t2hash.get_unchecked(i2) == 0
          && ((*t2hash.get_unchecked(i0) == 0 && *t2count.get_unchecked(i0) == 8)
            || (*t2hash.get_unchecked(i1) == 0 && *t2count.get_unchecked(i1) == 8)
            || (*t2hash.get_unchecked(i2) == 0 && *t2count.get_unchecked(i2) == 8))
        {
          duplicates += 1;
          *t2count.get_unchecked_mut(i0) -= 4;
          *t2hash.get_unchecked_mut(i0) ^= hash;
          *t2count.get_unchecked_mut(i1) -= 4;
          *t2count.get_unchecked_mut(i1) ^= 1;
          *t2hash.get_unchecked_mut(i1) ^= hash;
          *t2count.get_unchecked_mut(i2) -= 4;
          *t2count.get_unchecked_mut(i2) ^= 2;
          *t2hash.get_unchecked_mut(i2) ^= hash;
        }
        error = *t2count.get_unchecked(i0) < 4
          || *t2count.get_unchecked(i1) < 4
          || *t2count.get_unchecked(i2) < 4;
      }
      if error {
        break;
      }
    }

    if !error {
      let mut qsize = 0;
      for (i, &count) in t2count.iter().enumerate() {
        if (count >> 2) == 1 {
          unsafe {
            *alone.get_unchecked_mut(qsize) = i as u32;
          }
          qsize += 1;
        }
      }

      let mut stack_size = 0;
      let mut h012 = [0u32; 6];
      while qsize > 0 {
        qsize -= 1;
        let index = unsafe { *alone.get_unchecked(qsize) as usize };
        if unsafe { (*t2count.get_unchecked(index) >> 2) == 1 } {
          let hash = unsafe { *t2hash.get_unchecked(index) };
          let found = unsafe { *t2count.get_unchecked(index) & 3 };
          unsafe {
            *reverse_h.get_unchecked_mut(stack_size) = found;
            *reverse_order.get_unchecked_mut(stack_size) = hash;
          }
          stack_size += 1;

          let (i0, i1, i2) = hash_of_hash(hash, seg_len, seg_len_mask, seg_count_len);
          h012[1] = i1;
          h012[2] = i2;
          h012[3] = i0;
          h012[4] = i1;

          for offset in 1..=2 {
            let other = h012[(found + offset) as usize] as usize;
            unsafe {
              if (*t2count.get_unchecked(other) >> 2) == 2 {
                *alone.get_unchecked_mut(qsize) = other as u32;
                qsize += 1;
              }
              *t2count.get_unchecked_mut(other) -= 4;
              *t2count.get_unchecked_mut(other) ^= mod3(found + offset);
              *t2hash.get_unchecked_mut(other) ^= hash;
            }
          }
        }
      }

      if stack_size + duplicates == size {
        for i in (0..stack_size).rev() {
          let hash = unsafe { *reverse_order.get_unchecked(i) };
          let f_hash = T::from_hash(hash);
          let (i0, i1, i2) = hash_of_hash(hash, seg_len, seg_len_mask, seg_count_len);
          let found = unsafe { *reverse_h.get_unchecked(i) } as usize;
          h012[0] = i0;
          h012[1] = i1;
          h012[2] = i2;
          h012[3] = i0;
          h012[4] = i1;
          unsafe {
            *fingerprints.get_unchecked_mut(h012[found] as usize) = f_hash
              ^ *fingerprints.get_unchecked(h012[found + 1] as usize)
              ^ *fingerprints.get_unchecked(h012[found + 2] as usize);
          }
        }
        return (
          Desc {
            seed: current_seed,
            seg_len,
            seg_len_mask,
            seg_count_len,
          },
          fingerprints,
        );
      }
    }

    reverse_order[..size].fill(0);
    t2count.fill(0);
    t2hash.fill(0);
  }
  // Important: The mathematical principles of Binary Fuse filters require that all input keys must be **unique**.
  // If the input data contains duplicates, the fingerprint equation system will most likely have circular dependencies,
  // causing construction to fail to converge.
  //
  // Solution:
  // 1. Manually `sort()` and `dedup()` the key set.
  // 2. Use the `crate::bf::Bf` wrapper, which automatically handles deduplication before construction.
  //
  // 重要提示：Binary Fuse 过滤器的数学原理要求所有输入键必须是【唯一】的。
  // 如果输入数据中包含重复项，指纹方程组极大概率会出现循环依赖，导致构造无法收敛。
  //
  // 解决方案：
  // 1. 手动对键集进行 `sort()` 和 `dedup()`。
  // 2. 使用 `crate::bf::Bf` 包装器，它会自动处理构造前的去重逻辑。
  panic!(
    "binary fuse filter ( build failed ) :  duplicate keys not allowed",
  );
}
