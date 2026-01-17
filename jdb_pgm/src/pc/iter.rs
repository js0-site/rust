//! Iterator implementations for Pc.
//! Pc 迭代器实现。

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



    // Local helper for bit unpacking
    let mut word_idx = bit_off / 64;
    let mut bit_pos = (bit_off % 64) as u8;
    let residuals = &self.pc.residuals;
    let mask = if w < 64 { (1u64 << w) - 1 } else { !0u64 };

    // Standardized next_bits! macro with no loop dependency on `val`'s mutability warning
    macro_rules! next_bits {
      () => {{
        if w == 0 {
          0
        } else {
          let val;
          if bit_pos + (w as u8) <= 64 {
            val = (unsafe { *residuals.get_unchecked(word_idx) } >> bit_pos) & mask;
            bit_pos += w as u8;
            if bit_pos == 64 {
              word_idx += 1;
              bit_pos = 0;
            }
          } else {
            let low = unsafe { *residuals.get_unchecked(word_idx) } >> bit_pos;
            let bits_from_low = 64 - bit_pos;
            word_idx += 1;
            let current_word = unsafe { *residuals.get_unchecked(word_idx) };
            let bits_from_high = (w as u8) - bits_from_low;
            // Use wrapping_shl to avoid potential overflow panic when shifting by 64
            let high =
              (current_word & ((1u64.wrapping_shl(bits_from_high as u32)) - 1)) << bits_from_low;
            val = low | high;
            bit_pos = bits_from_high;
          }
          val
        }
      }};
    }

    // Fast path: Single segment block and no exceptions
    // 快速路径：单段块且无异常值
    if is_single_seg && !has_ex {
      let seg = unsafe { self.pc.segments.get_unchecked(self.current_seg_idx) };
      let mut cur_fp = (start as u128 * seg.slope_fp as u128) as i128 + seg.intercept_fp as i128;
      let slope = seg.slope_fp as i128;

      let mut i = 0;
      // SIMD loop: process 4 elements at a time
      // SIMD 循环：每次处理 4 个元素
      while i + 4 <= count {
        // 1. Unpack 4 residuals sequentially (dependency on bit stream)
        // 1. 顺序解包 4 个残差（位流依赖）
        let r0 = next_bits!();
        let r1 = next_bits!();
        let r2 = next_bits!();
        let r3 = next_bits!();

        // 2. Parallel Zigzag decoding (Logical shift required)
        // 2. 并行 Zigzag 解码（需要逻辑移位）
        let code_vec = i64x4::new([r0 as i64, r1 as i64, r2 as i64, r3 as i64]);

        // Manual logical shift: (arithmetic shift >> 1) & mask
        // 手动实现逻辑左移：(算术左移 >> 1) & 掩码
        let shr1 = (code_vec >> 1) & i64x4::splat(0x7fff_ffff_ffff_ffff);

        // Negate mask: if (code & 1) is 1 then -1 (all 1s), if 0 then 0
        // 生成负数掩码：如果 (code & 1) 位是 1 则为 -1（全 1），否则为 0
        let neg_mask = i64x4::ZERO - (code_vec & i64x4::ONE);
        let res_vec = shr1 ^ neg_mask;

        // 3. Compute predictions (Scalar but optimized)
        // 3. 计算预测值（标量但优化）
        // Since cur_fp is 128-bit, we keep it scalar to maintain precision before reduction
        let p0 = (cur_fp >> 32) as i64;
        cur_fp += slope;
        let p1 = (cur_fp >> 32) as i64;
        cur_fp += slope;
        let p2 = (cur_fp >> 32) as i64;
        cur_fp += slope;
        let p3 = (cur_fp >> 32) as i64;
        cur_fp += slope;

        let pred_vec = i64x4::new([p0, p1, p2, p3]);

        // 4. Parallel Accumulate (Wrapping addition)
        // 4. 并行累加（溢出折返加法）
        let final_vec: i64x4 = pred_vec + res_vec;
        let final_arr = final_vec.to_array();

        unsafe {
          let ptr = self.buffer.as_mut_ptr().add(i);
          *ptr = final_arr[0] as u64;
          *ptr.add(1) = final_arr[1] as u64;
          *ptr.add(2) = final_arr[2] as u64;
          *ptr.add(3) = final_arr[3] as u64;
        }
        i += 4;
      }

      // Handle remaining elements
      // 处理剩余元素
      while i < count {
        let code = next_bits!();
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
                if bit_pos + (w as u8) <= 64 {
                  bit_pos += w as u8;
                  if bit_pos == 64 {
                    word_idx += 1;
                    bit_pos = 0;
                  }
                } else {
                  let bits_from_low = 64 - bit_pos;
                  word_idx += 1;
                  bit_pos = (w as u8) - bits_from_low;
                }
              }
              v
            } else {
              next_bits!()
            }
          } else {
            next_bits!()
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
