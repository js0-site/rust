//! 基础泛型 Binary Fuse 过滤器算法实现
//! Base generic Binary Fuse filter algorithm implementation.

use alloc::{boxed::Box, vec::Vec};

#[cfg(feature = "bitcode")]
use bitcode::{Decode, Encode};
use libm::round;

use crate::{
  DmaSerializable, Filter, FilterRef,
  hash::{Fingerprint, mix, rand},
  prelude::bfuse::{
    Desc, hash_of_hash, mod3, parse_bfuse_desc, seg_len, serialize_bfuse_desc,
    size_factor,
  },
};

/// 泛型 Binary Fuse 过滤器
/// Generic Binary Fuse filter.
#[cfg_attr(feature = "bitcode", derive(Decode, Encode))]
#[derive(Debug, Clone)]
pub struct BinaryFuse<T: Fingerprint> {
  /// 描述符
  /// Descriptor
  pub desc: Desc,
  /// 指纹数据
  /// Fingerprint data
  pub fingerprints: Box<[T]>,
}

impl<T: Fingerprint> Filter<u64> for BinaryFuse<T> {
  #[inline(always)]
  fn contains(&self, key: &u64) -> bool {
    contains_impl(
      key,
      self.desc.seed,
      &self.fingerprints,
      self.desc.seg_len,
      self.desc.seg_len_mask,
      self.desc.seg_count_len,
    )
  }

  #[inline(always)]
  fn len(&self) -> usize {
    self.fingerprints.len()
  }
}

impl<T: Fingerprint> BinaryFuse<T> {
  /// Constructs the filter from a key iterator.
  pub fn try_from_iterator<I>(keys: I) -> Self
  where
    I: ExactSizeIterator<Item = u64> + Clone,
  {
    let (desc, fingerprints) = build_impl(keys, 1000);
    Self {
      desc,
      fingerprints,
    }
  }
}

impl<T: Fingerprint> From<&[u64]> for BinaryFuse<T> {
  fn from(keys: &[u64]) -> Self {
    Self::try_from_iterator(keys.iter().copied())
  }
}

impl<T: Fingerprint> From<&Vec<u64>> for BinaryFuse<T> {
  fn from(v: &Vec<u64>) -> Self {
    Self::try_from_iterator(v.iter().copied())
  }
}

impl<T: Fingerprint> From<Vec<u64>> for BinaryFuse<T> {
  fn from(v: Vec<u64>) -> Self {
    Self::try_from_iterator(v.iter().copied())
  }
}

impl<T: Fingerprint> DmaSerializable for BinaryFuse<T> {
  const DESCRIPTOR_LEN: usize = Desc::DMA_LEN;

  fn dma_copy_desc_to(&self, out: &mut [u8]) {
    serialize_bfuse_desc(&self.desc, out)
  }

  fn dma_fingerprints(&self) -> &[u8] {
    T::as_bytes(&self.fingerprints)
  }
}

/// 泛型 Binary Fuse 引用过滤器
/// Generic Binary Fuse reference filter.
#[derive(Debug, Clone)]
pub struct BinaryFuseRef<'a, T: Fingerprint> {
  /// 描述符
  /// Descriptor
  pub desc: Desc,
  /// 指纹数据引用
  /// Fingerprint data reference
  pub fingerprints: &'a [T],
}

impl<'a, T: Fingerprint> Filter<u64> for BinaryFuseRef<'a, T> {
  #[inline(always)]
  fn contains(&self, key: &u64) -> bool {
    contains_impl(
      key,
      self.desc.seed,
      self.fingerprints,
      self.desc.seg_len,
      self.desc.seg_len_mask,
      self.desc.seg_count_len,
    )
  }

  #[inline(always)]
  fn len(&self) -> usize {
    self.fingerprints.len()
  }
}

impl<'a, T: Fingerprint> FilterRef<'a, u64> for BinaryFuseRef<'a, T> {
  const FINGERPRINT_ALIGNMENT: usize = T::ALIGN;

  fn from_dma(desc: &[u8], fingerprints: &'a [u8]) -> Self {
    Self {
      desc: parse_bfuse_desc(desc),
      fingerprints: T::from_bytes(fingerprints),
    }
  }
}

/// 计算过滤器的尺寸信息
/// Calculates the size information for the filter.
pub fn calculate_size(size: usize) -> (u32, u32, u32, u32, u32) {
  let arity = 3u32;
  let seg_len = seg_len(arity, size as u32).min(262144);
  let seg_len_mask = seg_len - 1;
  let size_factor = size_factor(arity, size as u32);
  let capacity = if size > 1 {
    round(size as f64 * size_factor) as u32
  } else {
    0
  };
  let init_seg_count = capacity.div_ceil(seg_len);
  let (fp_array_len, seg_count) = {
    let array_len = init_seg_count * seg_len;
    let seg_count = if array_len.div_ceil(seg_len) < arity {
      1
    } else {
      array_len.div_ceil(seg_len) - (arity - 1)
    };
    ((seg_count + arity - 1) * seg_len, seg_count)
  };
  (
    seg_len,
    seg_len_mask,
    fp_array_len,
    seg_count,
    seg_count * seg_len,
  )
}

/// 检查键是否存在
/// Checks if a key exists in the filter.
#[inline(always)]
fn contains_impl<T: Fingerprint>(
  key: &u64,
  seed: u64,
  fingerprints: &[T],
  seg_len: u32,
  seg_len_mask: u32,
  seg_count_len: u32,
) -> bool {
  let hash = mix(*key, seed);
  let mut f = T::from_hash(hash);
  let (h0, h1, h2) = hash_of_hash(hash, seg_len, seg_len_mask, seg_count_len);
  unsafe {
    f ^= *fingerprints.get_unchecked(h0 as usize)
      ^ *fingerprints.get_unchecked(h1 as usize)
      ^ *fingerprints.get_unchecked(h2 as usize);
  }
  f == T::default()
}

/// 核心构造算法
/// Core construction algorithm.
fn build_impl<T: Fingerprint, I>(keys: I, max_iter: usize) -> (Desc, Box<[T]>)
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
  let mut seed = 1;

  for _ in 0..max_iter {
    let current_seed = rand(&mut seed);
    for (i, p) in start_pos.iter_mut().enumerate() {
      *p = (((i as u64) * (size as u64)) >> block_bits) as usize;
    }

    for key in keys.clone() {
      let hash = mix(key, current_seed);
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
  panic!("Failed to construct binary fuse filter after {max_iter} iterations");
}
