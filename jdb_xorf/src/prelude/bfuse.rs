//! Binary Fuse 过滤器实现
//! Implements Binary Fuse filters.
// Port of https://github.com/FastFilter/xorfilter/blob/master/binaryfusefilter.go

use core::convert::TryInto;

#[cfg(feature = "bitcode")]
use bitcode::{Decode, Encode};
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
/// Binary Fuse descriptor
#[cfg_attr(feature = "bitcode", derive(Decode, Encode))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Desc {
  /// 随机种子
  /// Random seed
  pub seed: u64,
  /// 段长度
  /// Segment length
  pub seg_len: u32,
  /// 段长度掩码
  /// Segment length mask
  pub seg_len_mask: u32,
  /// 段计数长度
  /// Segment count length
  pub seg_count_len: u32,
}

impl Desc {
  pub const DMA_LEN: usize = u64::BITS as usize / 8 + (u32::BITS as usize / 8) * 3;
}

#[inline]
pub fn parse_bfuse_descriptor(descriptor: &[u8]) -> Desc {
  // Safe: descriptor is guaranteed to have at least DMA_LEN bytes
  // 安全：descriptor 保证至少有 DMA_LEN 字节
  Desc {
    seed: u64::from_le_bytes(descriptor[0..8].try_into().unwrap()),
    seg_len: u32::from_le_bytes(descriptor[8..12].try_into().unwrap()),
    seg_len_mask: u32::from_le_bytes(descriptor[12..16].try_into().unwrap()),
    seg_count_len: u32::from_le_bytes(descriptor[16..20].try_into().unwrap()),
  }
}

#[inline]
pub fn serialize_bfuse_descriptor(descriptor: &Desc, out: &mut [u8]) {
  out[0..8].copy_from_slice(&descriptor.seed.to_le_bytes());
  out[8..12].copy_from_slice(&descriptor.seg_len.to_le_bytes());
  out[12..16].copy_from_slice(&descriptor.seg_len_mask.to_le_bytes());
  out[16..20].copy_from_slice(&descriptor.seg_count_len.to_le_bytes());
}

/// Implements `try_from(&[u64])` for an binary fuse filter of fingerprint type `$fpty`.
#[doc(hidden)]
#[macro_export]
macro_rules! bfuse_from_impl(
    ($keys:ident fingerprint $fpty:ident, max iter $max_iter:expr) => {
        {
            use libm::round;
            use $crate::{
                fingerprint,
                make_block,
                make_fp_block,
                prelude::{
                    mix,
                    bfuse::{seg_len, size_factor, hash_of_hash, mod3},
                },
                splitmix64::splitmix64,
            };

            #[cfg(debug_assertions)] {
                use $crate::prelude::all_distinct;
                debug_assert!(all_distinct($keys.clone()), "Binary Fuse filters must be constructed from a collection containing all distinct keys.");
            }

            let arity = 3u32;
            let size: usize = $keys.len();
            let seg_len: u32 = seg_len(arity, size as u32).min(262144);
            let seg_len_mask: u32 = seg_len - 1;
            let size_factor: f64 = size_factor(arity, size as u32);
            let capacity: u32 = if size > 1 {
                round(size as f64 * size_factor) as u32
            } else { 0 };
            let init_seg_count = capacity.div_ceil(seg_len);
            let (fp_array_len, seg_count) = {
                let array_len = init_seg_count * seg_len;
                let seg_count: u32 = {
                    let proposed = array_len.div_ceil(seg_len);
                    if proposed < arity {
                        1
                    } else {
                        proposed - (arity - 1)
                    }
                };
                let array_len: u32 = (seg_count + arity - 1) * seg_len;
                (array_len as usize, seg_count)
            };
            let seg_count_len = seg_count * seg_len;

            let mut fingerprints: Box<[$fpty]> = make_fp_block!(fp_array_len, $fpty);

            let mut rng = 1;
            let mut seed = splitmix64(&mut rng);
            let capacity = fingerprints.len();
            let mut alone: Box<[u32]> = make_block!(with capacity sets);
            let mut t2count: Box<[u8]> = make_block!(with capacity sets);
            let mut t2hash: Box<[u64]> = make_block!(with capacity sets);
            let mut reverse_h: Box<[u8]> = make_block!(with size sets);
            let size_plus_1: usize = size + 1;
            let mut reverse_order: Box<[u64]> = make_block!(with size_plus_1 sets);
            reverse_order[size] = 1;

            let block_bits = {
                let mut block_bits = 1;
                while (1 << block_bits) < seg_count {
                    block_bits += 1;
                }
                block_bits
            };

            let start_pos_len: usize = 1 << block_bits;
            let mut start_pos: Box<[usize]> = make_block!(with start_pos_len sets);
            let mut h012: [u32; 6] = [0; 6];
            let mut done = false;
            let mut ultimate_size = 0;
            for _ in 0..$max_iter {
                for i in 0..start_pos_len {
                    start_pos[i] = (((i as u64) * (size as u64)) >> block_bits) as usize;
                }
                for key in $keys.clone() {
                    let hash = mix(key, seed);
                    let mut seg_index = hash >> (64 - block_bits);
                    while reverse_order[start_pos[seg_index as usize] as usize] != 0 {
                        seg_index += 1;
                        seg_index &= (1 << block_bits) - 1;
                    }
                    reverse_order[start_pos[seg_index as usize] as usize] = hash;
                    start_pos[seg_index as usize] += 1;
                }

                let mut error = false;
                let mut duplicates = 0;
                for i in 0..size {
                    let hash = unsafe { *reverse_order.get_unchecked(i) };
                    let (index1, index2, index3) = hash_of_hash(hash, seg_len, seg_len_mask, seg_count_len);
                    let (index1, index2, index3) = (index1 as usize, index2 as usize, index3 as usize);
                    unsafe {
                        *t2count.get_unchecked_mut(index1) += 4;
                        // t2count[index1] ^= 0; NOOP
                        *t2hash.get_unchecked_mut(index1) ^= hash;
                        *t2count.get_unchecked_mut(index2) += 4;
                        *t2count.get_unchecked_mut(index2) ^= 1;
                        *t2hash.get_unchecked_mut(index2) ^= hash;
                        *t2count.get_unchecked_mut(index3) += 4;
                        *t2count.get_unchecked_mut(index3) ^= 2;
                        *t2hash.get_unchecked_mut(index3) ^= hash;

                        if *t2hash.get_unchecked(index1) & *t2hash.get_unchecked(index2) & *t2hash.get_unchecked(index3) == 0 {
                            if ((*t2hash.get_unchecked(index1) == 0) && (*t2count.get_unchecked(index1) == 8)) ||
                               ((*t2hash.get_unchecked(index2) == 0) && (*t2count.get_unchecked(index2) == 8)) ||
                               ((*t2hash.get_unchecked(index3) == 0) && (*t2count.get_unchecked(index3) == 8)) {
                                    duplicates += 1;
                                    *t2count.get_unchecked_mut(index1) -= 4;
                                    *t2hash.get_unchecked_mut(index1) ^= hash;
                                    *t2count.get_unchecked_mut(index2) -= 4;
                                    *t2count.get_unchecked_mut(index2) ^= 1;
                                    *t2hash.get_unchecked_mut(index2) ^= hash;
                                    *t2count.get_unchecked_mut(index3) -= 4;
                                    *t2count.get_unchecked_mut(index3) ^= 2;
                                    *t2hash.get_unchecked_mut(index3) ^= hash;
                            }
                        }
                        error = *t2count.get_unchecked(index1) < 4 || *t2count.get_unchecked(index2) < 4 || *t2count.get_unchecked(index3) < 4;
                    }
                }
                if error {
                    continue;
                }

                // Key addition complete. Perform enqueing.

                let mut qsize = 0;
                for i in 0..capacity {
                    unsafe {
                        *alone.get_unchecked_mut(qsize) = i as u32;
                        if (*t2count.get_unchecked(i) >> 2) == 1 {
                            qsize += 1;
                        }
                    }
                }
                let mut stack_size = 0;
                while qsize > 0 {
                    qsize -= 1;
                    let index = unsafe { *alone.get_unchecked(qsize) as usize };
                    if unsafe { (*t2count.get_unchecked(index) >> 2) == 1 } {
                        let hash = unsafe { *t2hash.get_unchecked(index) };
                        let found: u8 = unsafe { *t2count.get_unchecked(index) & 3 };
                        unsafe {
                            *reverse_h.get_unchecked_mut(stack_size) = found;
                            *reverse_order.get_unchecked_mut(stack_size) = hash;
                        }
                        stack_size += 1;

                        let (index1, index2, index3) = hash_of_hash(hash, seg_len, seg_len_mask, seg_count_len);

                        h012[1] = index2;
                        h012[2] = index3;
                        h012[3] = index1;
                        h012[4] = h012[1];

                        let other_index1 = h012[(found + 1) as usize] as usize;
                        unsafe {
                            *alone.get_unchecked_mut(qsize) = other_index1 as u32;
                            if (*t2count.get_unchecked(other_index1) >> 2) == 2 {
                                qsize += 1;
                            }
                            *t2count.get_unchecked_mut(other_index1) -= 4;
                            *t2count.get_unchecked_mut(other_index1) ^= mod3(found + 1);
                            *t2hash.get_unchecked_mut(other_index1) ^= hash;
                        }

                        let other_index2 = h012[(found + 2) as usize] as usize;
                        unsafe {
                            *alone.get_unchecked_mut(qsize) = other_index2 as u32;
                            if (*t2count.get_unchecked(other_index2) >> 2) == 2 {
                                qsize += 1;
                            }
                            *t2count.get_unchecked_mut(other_index2) -= 4;
                            *t2count.get_unchecked_mut(other_index2) ^= mod3(found + 2);
                            *t2hash.get_unchecked_mut(other_index2) ^= hash;
                        }
                    }
                }

                if stack_size + duplicates == size {
                    ultimate_size = stack_size;
                    done = true;
                    break
                }

                // Filter failed to be created; reset for a retry.
                for i in 0..size {
                    reverse_order[i] = 0;
                }
                for i in 0..capacity {
                    t2count[i] = 0;
                    t2hash[i] = 0;
                }
                seed = splitmix64(&mut rng)
            }
            if !done {
                return Err($crate::error::Error::ConstructionFailed);
            }

            // 构造所有指纹
            // Construct all fingerprints
            let size = ultimate_size;
            for i in (0..size).rev() {
                let hash = unsafe { *reverse_order.get_unchecked(i) };
                let xor2 = (fingerprint!(hash) as $fpty);
                let (index1, index2, index3) = hash_of_hash(hash, seg_len, seg_len_mask, seg_count_len);
                let found = unsafe { *reverse_h.get_unchecked(i) } as usize;
		            h012[0] = index1;
		            h012[1] = index2;
		            h012[2] = index3;
		            h012[3] = h012[0];
		            h012[4] = h012[1];
                unsafe {
		            *fingerprints.get_unchecked_mut(h012[found] as usize) =
                      xor2
                    ^ *fingerprints.get_unchecked(h012[found + 1] as usize)
                    ^ *fingerprints.get_unchecked(h012[found + 2] as usize);
                }
            }

            Ok(Self {
                descriptor: Desc{seed,
                seg_len,
                seg_len_mask,
                seg_count_len,},
                fingerprints,
            })
        }
    };
);

/// Implements `contains(u64)` for a binary fuse filter of fingerprint type `$fpty`.
#[doc(hidden)]
#[macro_export]
macro_rules! bfuse_contains_impl(
    ($key:expr, $self:expr, fingerprint $fpty:ident) => {
        {
            use $crate::{
                fingerprint,
                prelude::{
                    mix,
                    bfuse::hash_of_hash
                },
            };
            let hash = mix($key, $self.descriptor.seed);
            let mut f = fingerprint!(hash) as $fpty;
            let (h0, h1, h2) = hash_of_hash(hash, $self.descriptor.seg_len, $self.descriptor.seg_len_mask, $self.descriptor.seg_count_len);
            unsafe {
                f ^= *$self.fingerprints.get_unchecked(h0 as usize)
                   ^ *$self.fingerprints.get_unchecked(h1 as usize)
                   ^ *$self.fingerprints.get_unchecked(h2 as usize);
            }
            f == 0
        }
    };
);
