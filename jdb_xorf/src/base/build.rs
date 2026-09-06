use alloc::boxed::Box;

use super::calc::SizeInfo;
use crate::{
  Error, Result,
  hash::{Fingerprint, mix_key},
  prelude::bfuse::{Desc, hash_of_hash, mod3},
};

/// SplitMix64 PRNG for seed generation (matches Go reference)
/// 用于种子生成的 SplitMix64 伪随机数生成器（与 Go 参考实现一致）
#[inline(always)]
fn splitmix64(seed: &mut u64) -> u64 {
  *seed = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
  let mut z = *seed;
  z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
  z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
  z ^ (z >> 31)
}

/// 尝试构造过滤器，返回 Result
/// Try constructing the filter, returning Result.
pub fn try_make<T: Fingerprint>(keys: &[u64], max_iter: usize) -> Result<(Desc, Box<[T]>)> {
  #[cfg(debug_assertions)]
  {
    use crate::prelude::all_distinct;
    if !all_distinct(keys.iter().copied()) {
      return Err(Error::DuplicateKeys);
    }
  }

  let size = keys.len();
  if size == 0 {
    return Ok((Desc::default(), Box::new([])));
  }

  let SizeInfo {
    seg_len,
    seg_len_mask,
    fp_array_len,
    seg_count,
    seg_count_len,
  } = SizeInfo::from_size(size);

  // Mutable copies for dynamic segment length adjustment
  // 用于动态段长度调整的可变副本
  let mut cur_seg_len = seg_len;
  let mut cur_seg_len_mask = seg_len_mask;
  let mut cur_seg_count = seg_count;
  let mut cur_seg_count_len = seg_count_len;

  // Initialize fingerprint array
  // 初始化指纹数组
  let mut fingerprints: Box<[T]> = make_fp_block!(fp_array_len, T);

  let mut alone: Box<[u32]> = vec![0; fp_array_len as usize].into_boxed_slice();
  let mut t2count: Box<[u8]> = vec![0; fp_array_len as usize].into_boxed_slice();
  let mut t2hash: Box<[u64]> = vec![0; fp_array_len as usize].into_boxed_slice();
  let mut reverse_h: Box<[u8]> = vec![0; size].into_boxed_slice();
  let mut reverse_order: Box<[u64]> = vec![0; size + 1].into_boxed_slice();
  reverse_order[size] = 1;

  // Pre-allocate start_pos large enough for both normal and halved segment lengths
  // 预分配 start_pos，确保足够容纳正常和减半段长度
  let max_block_bits = (seg_count * 2 + 2)
    .next_power_of_two()
    .trailing_zeros()
    .max(1);
  let mut rng_counter: u64 = 1;
  let mut start_pos: Box<[usize]> = vec![0; 1 << max_block_bits].into_boxed_slice();

  for iter_n in 0..max_iter {
    // Dynamic segment length adjustment for problematic sizes (port from Go reference)
    // 针对问题尺寸的动态段长度调整（移植自 Go 参考实现）
    if size > 4 && size < 1_000_000 {
      match iter_n % 4 {
        2 => {
          // Switch to smaller segment size
          // 切换到更小的段长度
          cur_seg_len = seg_len / 2;
          cur_seg_len_mask = cur_seg_len - 1;
          cur_seg_count = seg_count * 2 + 2;
          cur_seg_count_len = cur_seg_count * cur_seg_len;
        }
        3 => {
          // Restore the calculated segment size
          // 恢复计算得到的段长度
          cur_seg_len = seg_len;
          cur_seg_len_mask = seg_len_mask;
          cur_seg_count = seg_count;
          cur_seg_count_len = seg_count_len;
        }
        _ => {}
      }
    }

    let block_bits = cur_seg_count.next_power_of_two().trailing_zeros().max(1);
    let mask = (1usize << block_bits) - 1;

    let current_seed = splitmix64(&mut rng_counter);
    for (i, p) in start_pos[..1 << block_bits].iter_mut().enumerate() {
      *p = (((i as u64) * (size as u64)) >> block_bits) as usize;
    }

    for &key in keys {
      let hash = mix_key(key, current_seed);
      let mut seg_index = (hash >> (64 - block_bits)) as usize;
      unsafe {
        let mut pos = *start_pos.get_unchecked(seg_index);
        while *reverse_order.get_unchecked(pos) != 0 {
          seg_index = (seg_index + 1) & mask;
          pos = *start_pos.get_unchecked(seg_index);
        }
        *reverse_order.get_unchecked_mut(pos) = hash;
        *start_pos.get_unchecked_mut(seg_index) = pos + 1;
      }
    }

    let mut error = false;
    let mut duplicates = 0;
    for &hash in reverse_order.iter().take(size) {
      let (i0, i1, i2) = hash_of_hash(hash, cur_seg_len, cur_seg_len_mask, cur_seg_count_len);
      let (i0, i1, i2) = (i0 as usize, i1 as usize, i2 as usize);
      unsafe {
        update_buckets(&mut t2count, &mut t2hash, (i0, i1, i2), hash, true);

        if *t2hash.get_unchecked(i0) & *t2hash.get_unchecked(i1) & *t2hash.get_unchecked(i2) == 0
          && (is_dup_slot(&t2hash, &t2count, i0)
            || is_dup_slot(&t2hash, &t2count, i1)
            || is_dup_slot(&t2hash, &t2count, i2))
        {
          duplicates += 1;
          update_buckets(&mut t2count, &mut t2hash, (i0, i1, i2), hash, false);
        }
        error |= *t2count.get_unchecked(i0) < 4
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

          let (i0, i1, i2) = hash_of_hash(hash, cur_seg_len, cur_seg_len_mask, cur_seg_count_len);
          let h012 = [i0, i1, i2, i0, i1];

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
          let (i0, i1, i2) = hash_of_hash(hash, cur_seg_len, cur_seg_len_mask, cur_seg_count_len);
          let found = unsafe { *reverse_h.get_unchecked(i) } as usize;
          let h012 = [i0, i1, i2, i0, i1];
          unsafe {
            *fingerprints.get_unchecked_mut(h012[found] as usize) = f_hash
              ^ *fingerprints.get_unchecked(h012[found + 1] as usize)
              ^ *fingerprints.get_unchecked(h012[found + 2] as usize);
          }
        }
        return Ok((
          Desc {
            seed: current_seed,
            seg_len: cur_seg_len,
            seg_len_mask: cur_seg_len_mask,
            seg_count_len: cur_seg_count_len,
          },
          fingerprints,
        ));
      }
    }

    reverse_order[..size].fill(0);
    t2count.fill(0);
    t2hash.fill(0);
  }

  Err(Error::BuildFailed)
}

/// 核心构造算法（失败时返回空过滤器，避免 panic）
/// Core construction algorithm (returns an empty filter on failure to avoid panicking).
pub fn make<T: Fingerprint>(keys: &[u64], max_iter: usize) -> (Desc, Box<[T]>) {
  try_make(keys, max_iter).unwrap_or_else(|_| (Desc::default(), Box::new([])))
}

#[inline(always)]
unsafe fn update_buckets(
  t2count: &mut [u8],
  t2hash: &mut [u64],
  (i0, i1, i2): (usize, usize, usize),
  hash: u64,
  add: bool,
) {
  unsafe {
    if add {
      *t2count.get_unchecked_mut(i0) += 4;
      *t2count.get_unchecked_mut(i1) += 4;
      *t2count.get_unchecked_mut(i2) += 4;
    } else {
      *t2count.get_unchecked_mut(i0) -= 4;
      *t2count.get_unchecked_mut(i1) -= 4;
      *t2count.get_unchecked_mut(i2) -= 4;
    }
    *t2count.get_unchecked_mut(i1) ^= 1;
    *t2count.get_unchecked_mut(i2) ^= 2;
    *t2hash.get_unchecked_mut(i0) ^= hash;
    *t2hash.get_unchecked_mut(i1) ^= hash;
    *t2hash.get_unchecked_mut(i2) ^= hash;
  }
}

#[inline(always)]
unsafe fn is_dup_slot(t2hash: &[u64], t2count: &[u8], idx: usize) -> bool {
  unsafe { *t2hash.get_unchecked(idx) == 0 && *t2count.get_unchecked(idx) == 8 }
}
