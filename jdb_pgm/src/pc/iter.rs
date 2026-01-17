//! Iterator implementations for Pc.
//! Pc 迭代器实现。

use wide::{i64x4, u64x4};

use super::{
  PcBase,
  types::{FLAG_HAS_EX, FLAG_SINGLE_SEG},
};
use crate::util::bits::read_bits;

/// Forward iterator for Pc.
/// Pc 正向迭代器。
pub struct PcIterBase<'a, const B: usize> {
  pub(crate) pc: &'a PcBase<B>,
  pub(crate) current_idx: usize,
  pub(crate) current_seg_idx: usize,
  pub(crate) current_block_idx: isize,
  pub(crate) buffer: Vec<u64>,
  pub(crate) buffer_pos: u16,
  pub(crate) buffer_len: u16,
}

impl<const B: usize> Iterator for PcIterBase<'_, B> {
  type Item = u64;

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    if self.buffer_pos < self.buffer_len {
      let val = unsafe { *self.buffer.get_unchecked(self.buffer_pos as usize) };
      self.buffer_pos += 1;
      self.current_idx += 1;
      return Some(val);
    }
    if self.current_idx >= self.pc.len {
      return None;
    }
    self.refill_buffer();
    if self.buffer_pos < self.buffer_len {
      let val = unsafe { *self.buffer.get_unchecked(self.buffer_pos as usize) };
      self.buffer_pos += 1;
      self.current_idx += 1;
      Some(val)
    } else {
      None
    }
  }
}

impl<const B: usize> PcIterBase<'_, B> {
  /// Refill buffer with next block's decoded values.
  /// 用下一个块的解码值填充缓冲区。
  #[inline(never)]
  fn refill_buffer(&mut self) {
    let n = self.pc.len;
    self.current_block_idx += 1;
    let b_idx = self.current_block_idx as usize;
    if b_idx >= self.pc.block_meta.len() {
      self.buffer_len = 0;
      return;
    }

    let meta = unsafe { self.pc.block_meta.get_unchecked(b_idx) };
    let start = b_idx * B;
    let count = (n - start).min(B);
    let w = meta.bit_width as usize;
    let bit_off = meta.bit_offset as usize;
    let ex_offset = meta.exception_offset as usize;

    let info = meta.seg_idx;
    let has_ex = (info & FLAG_HAS_EX) != 0;
    let is_single_seg = (info & FLAG_SINGLE_SEG) != 0;

    let res_ptr = self.pc.residuals.as_ptr() as *const u8;
    // We can rely on padding in BitWriter ensuring we don't read OOB even at the last element.
    // 我们依赖 BitWriter 中的填充确保即使在最后一个元素也不会读取越界。

    // Fast path: Single segment block and no exceptions
    // 快速路径：单段块且无异常值
    if is_single_seg && !has_ex {
      let seg = unsafe { self.pc.segments.get_unchecked(self.current_seg_idx) };
      let mut cur_fp = (start as u128 * seg.slope_fp as u128) as i128 + seg.intercept_fp as i128;
      let slope = seg.slope_fp as i128;
      let slope_u64 = seg.slope_fp;
      // Precompute slope sequence for SIMD: [0, S, 2S, 3S]
      let slope_seq = u64x4::new([
        0,
        slope_u64,
        slope_u64.wrapping_mul(2),
        slope_u64.wrapping_mul(3),
      ]);
      let slope_step = slope_u64.wrapping_mul(4);
      let (slope3, slope_overflow) = slope_u64.overflowing_mul(3);
      let max_safe_low = if !slope_overflow {
        u64::MAX - slope3
      } else {
        0
      };
      let allow_fast = !slope_overflow;

      let mask = if w < 64 { (1u64 << w) - 1 } else { !0u64 };
      let mut global_bit_pos = bit_off;

      let mut i = 0;
      // SIMD loop: process 4 elements at a time.
      // SIMD 循环：每次处理 4 个元素。
      while i + 4 <= count {
        // 1. Unpack 4 residuals using Unaligned Byte Reads
        let r0;
        let r1;
        let r2;
        let r3;

        if w <= 14 {
          // Super-Fast path: Batch read 4 residuals (4*14 + 7 <= 63)
          // valid for w <= 14.
          unsafe {
            let ptr = res_ptr.add(global_bit_pos >> 3);
            let shift = (global_bit_pos & 7) as u32;
            // Read 64 bits containing all 4 residuals
            let val = ptr.cast::<u64>().read_unaligned() >> shift;

            r0 = val & mask;
            r1 = (val >> w) & mask;
            r2 = (val >> (2 * w)) & mask;
            r3 = (val >> (3 * w)) & mask;

            global_bit_pos += 4 * w;
          }
        } else if w <= 56 {
          // Optimized unaligned read (valid for w <= 64 - 7)
          unsafe {
            let ptr = res_ptr.add(global_bit_pos >> 3);
            let shift = (global_bit_pos & 7) as u32;
            r0 = (ptr.cast::<u64>().read_unaligned() >> shift) & mask;
            global_bit_pos += w;

            let ptr = res_ptr.add(global_bit_pos >> 3);
            let shift = (global_bit_pos & 7) as u32;
            r1 = (ptr.cast::<u64>().read_unaligned() >> shift) & mask;
            global_bit_pos += w;

            let ptr = res_ptr.add(global_bit_pos >> 3);
            let shift = (global_bit_pos & 7) as u32;
            r2 = (ptr.cast::<u64>().read_unaligned() >> shift) & mask;
            global_bit_pos += w;

            let ptr = res_ptr.add(global_bit_pos >> 3);
            let shift = (global_bit_pos & 7) as u32;
            r3 = (ptr.cast::<u64>().read_unaligned() >> shift) & mask;
            global_bit_pos += w;
          }
        } else {
          // Fallback for large w (rare)
          r0 = read_bits(&self.pc.residuals, global_bit_pos, w as u8);
          global_bit_pos += w;
          r1 = read_bits(&self.pc.residuals, global_bit_pos, w as u8);
          global_bit_pos += w;
          r2 = read_bits(&self.pc.residuals, global_bit_pos, w as u8);
          global_bit_pos += w;
          r3 = read_bits(&self.pc.residuals, global_bit_pos, w as u8);
          global_bit_pos += w;
        }

        // 2. Parallel Zigzag decoding (Logical shift required).
        // 2. 并行 Zigzag 解码（需要逻辑移位）。
        let code_vec = i64x4::new([r0 as i64, r1 as i64, r2 as i64, r3 as i64]);

        let shr1 = (code_vec >> 1) & i64x4::splat(0x7fff_ffff_ffff_ffff);
        let neg_mask = i64x4::ZERO - (code_vec & i64x4::ONE);
        let res_vec = shr1 ^ neg_mask;

        let fp_low = cur_fp as u64;

        // Check if SIMD addition is safe (no carry/wrap within the vector)
        if allow_fast && fp_low <= max_safe_low {
          // Fast Path: Full SIMD Prediction
          let fp_high = (cur_fp >> 64) as i64; // Sign-extended high part
          let high_val = (fp_high as u64) << 32;
          let h_vec = u64x4::splat(high_val);
          let l_base = u64x4::splat(fp_low);

          // [L, L+S, L+2S, L+3S]
          let l_vec = l_base + slope_seq;

          // pred = (H << 32) | (L >> 32)
          let p_vec_u64 = h_vec | (l_vec >> 32);

          // Reinterpret as i64x4 for addition with residuals
          let pred_vec: i64x4 = unsafe { std::mem::transmute(p_vec_u64) };

          let final_vec: i64x4 = pred_vec + res_vec;
          let final_arr = final_vec.to_array();

          unsafe {
            let ptr = self.buffer.as_mut_ptr().add(i);
            *ptr = final_arr[0] as u64;
            *ptr.add(1) = final_arr[1] as u64;
            *ptr.add(2) = final_arr[2] as u64;
            *ptr.add(3) = final_arr[3] as u64;
          }

          // Update state
          cur_fp += slope_step as i128;
          // Check if high part needs update (via carry check on low part)
          // Actually cur_fp += slope_step handles it correctly.
        } else {
          // Slow Path: Scalar 128-bit math
          let p0 = (cur_fp >> 32) as i64;
          cur_fp += slope;
          let p1 = (cur_fp >> 32) as i64;
          cur_fp += slope;
          let p2 = (cur_fp >> 32) as i64;
          cur_fp += slope;
          let p3 = (cur_fp >> 32) as i64;
          cur_fp += slope;

          let pred_vec = i64x4::new([p0, p1, p2, p3]);
          let final_vec: i64x4 = pred_vec + res_vec;
          let final_arr = final_vec.to_array();

          unsafe {
            let ptr = self.buffer.as_mut_ptr().add(i);
            *ptr = final_arr[0] as u64;
            *ptr.add(1) = final_arr[1] as u64;
            *ptr.add(2) = final_arr[2] as u64;
            *ptr.add(3) = final_arr[3] as u64;
          }
        }
        i += 4;
      }

      // Handle remaining elements
      // 处理剩余元素
      // Handle remaining elements
      // 处理剩余元素
      while i < count {
        let code = if w == 0 {
          0
        } else {
          let val = read_bits(&self.pc.residuals, global_bit_pos, w as u8);
          global_bit_pos += w;
          val
        };
        let res = (code >> 1) as i64 ^ -((code & 1) as i64);
        unsafe {
          *self.buffer.get_unchecked_mut(i) = ((cur_fp >> 32) as i64 + res) as u64;
        }
        cur_fp += slope;
        i += 1;
      }
    } else {
      // General path: Handles segment crossing or exceptions
      // 通用路径：处理跨段或异常值
      let mut i = 0;
      let mut ex_idx = ex_offset;
      let mut global_bit_pos = bit_off;

      while i < count {
        let g_idx = start + i;
        while self.current_seg_idx + 1 < self.pc.segments.len()
          && g_idx >= self.pc.segments[self.current_seg_idx + 1].start_idx as usize
        {
          self.current_seg_idx += 1;
        }
        let seg = unsafe { self.pc.segments.get_unchecked(self.current_seg_idx) };
        let sub_count = if self.current_seg_idx + 1 < self.pc.segments.len() {
          (self.pc.segments[self.current_seg_idx + 1].start_idx as usize - g_idx).min(count - i)
        } else {
          count - i
        };

        let mut cur_fp = (g_idx as u128 * seg.slope_fp as u128) as i128 + seg.intercept_fp as i128;
        let slope = seg.slope_fp as i128;

        for j in 0..sub_count {
          let code = if has_ex {
            let bm_idx = (g_idx + j) / 64;
            let bm_bit = (g_idx + j) % 64;
            let is_ex = (unsafe { *self.pc.bitmap.get_unchecked(bm_idx) } >> bm_bit) & 1 == 1;

            if is_ex {
              let v = unsafe { *self.pc.exceptions.get_unchecked(ex_idx) };
              ex_idx += 1;
              if w > 0 {
                // Skip the bits in residuals.
                // 跳过残差位。
                global_bit_pos += w;
              }
              v
            } else if w > 0 {
              let val = read_bits(&self.pc.residuals, global_bit_pos, w as u8);
              global_bit_pos += w;
              val
            } else {
              0
            }
          } else if w > 0 {
            let val = read_bits(&self.pc.residuals, global_bit_pos, w as u8);
            global_bit_pos += w;
            val
          } else {
            0
          };

          let res = (code >> 1) as i64 ^ -((code & 1) as i64);
          unsafe {
            *self.buffer.get_unchecked_mut(i + j) = ((cur_fp >> 32) as i64 + res) as u64;
          }
          cur_fp += slope;
        }
        i += sub_count;
      }
    }

    self.buffer_pos = 0;
    self.buffer_len = count as u16;
  }
}

/// Reverse iterator for Pc
/// Pc 逆向迭代器
pub struct PcRevIterBase<'a, const B: usize> {
  pub(crate) pc: &'a PcBase<B>,
  pub(crate) current_idx: isize,
  pub(crate) current_seg_idx: isize,
}

impl<const B: usize> Iterator for PcRevIterBase<'_, B> {
  type Item = u64;

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    if self.current_idx < 0 {
      return None;
    }
    let idx = self.current_idx as usize;
    let b_idx = idx / B;
    let meta = unsafe { self.pc.block_meta.get_unchecked(b_idx) };

    // Backtrack segment if needed
    // 必要时回退到上一个段
    while self.current_seg_idx > 0
      && (idx as u32) < self.pc.segments[self.current_seg_idx as usize].start_idx
    {
      self.current_seg_idx -= 1;
    }
    let seg = unsafe {
      self
        .pc
        .segments
        .get_unchecked(self.current_seg_idx as usize)
    };
    let pred =
      (((idx as u128 * seg.slope_fp as u128) as i128 + seg.intercept_fp as i128) >> 32) as i64;

    let w = meta.bit_width as usize;

    // Check if block has exceptions via flag bit
    // 通过标志位判断是否有异常值
    let has_ex = (meta.seg_idx & FLAG_HAS_EX) != 0;

    let code = if !has_ex {
      if w == 0 {
        0
      } else {
        read_bits(
          &self.pc.residuals,
          meta.bit_offset as usize + (idx % B) * w,
          w as u8,
        )
      }
    } else {
      let bm_idx = idx / 64;
      let bm_bit = idx % 64;
      let is_ex = (unsafe { *self.pc.bitmap.get_unchecked(bm_idx) } >> bm_bit) & 1 == 1;
      if is_ex {
        // Count exceptions before this position within the block
        // 计算块内此位置之前的异常值数量
        let start_word = (b_idx * B) / 64;
        let mut rank = 0u32;
        for i in start_word..bm_idx {
          rank += unsafe { self.pc.bitmap.get_unchecked(i).count_ones() };
        }
        rank +=
          (unsafe { *self.pc.bitmap.get_unchecked(bm_idx) } & ((1u64 << bm_bit) - 1)).count_ones();
        unsafe {
          *self
            .pc
            .exceptions
            .get_unchecked(meta.exception_offset as usize + rank as usize)
        }
      } else if w == 0 {
        0
      } else {
        read_bits(
          &self.pc.residuals,
          meta.bit_offset as usize + (idx % B) * w,
          w as u8,
        )
      }
    };

    self.current_idx -= 1;
    let res = (code >> 1) as i64 ^ -((code & 1) as i64);
    Some(pred.wrapping_add(res) as u64)
  }
}
