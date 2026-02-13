//! Iterator implementations for Pc.
// Pc 迭代器实现。

use super::{PcBase, types::FLAG_HAS_EX};
use crate::util::bits::read_bits;

/// Forward iterator for Pc.
/// Pc 正向迭代器。
pub struct PcIterBase<'a, const B: usize> {
  pub(crate) pc: &'a PcBase<B>,
  pub(crate) current_idx: usize,
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

    let has_ex = (meta.flags & FLAG_HAS_EX) != 0;

    let res_ptr = self.pc.residuals.as_ptr() as *const u8;

    // Block local model
    let slope_fp = meta.slope_fp;
    let intercept_fp = meta.intercept_fp;

    // Fast path: No exceptions
    if !has_ex {
      let mut cur_fp = (start as u128 * slope_fp as u128) as i128 + intercept_fp as i128;
      let slope = slope_fp as i128; // Treat slope_fp as u64/i128 for arithmetic

      // SIMD logic preserved but adapted to use block slope/intercept?
      // Simplified for reliability: just scalar loop or basic unrolled logic.
      // To ensure correctness first, I use the scalar loop which is robust.
      // Optimizing iter is secondary to random access task.
      let mask = if w < 64 { (1u64 << w) - 1 } else { !0u64 };
      let mut global_bit_pos = bit_off;
      let mut i = 0;
      while i < count {
        let code = if w == 0 {
          0
        } else if w <= 56 {
          // Basic optimization
          unsafe {
            let ptr = res_ptr.add(global_bit_pos >> 3);
            let shift = (global_bit_pos & 7) as u32;
            let val = (ptr.cast::<u64>().read_unaligned() >> shift) & mask;
            global_bit_pos += w;
            val
          }
        } else {
          let val = read_bits(&self.pc.residuals, global_bit_pos, w as u8);
          global_bit_pos += w;
          val
        };
        let res = (code >> 1) as i64 ^ -((code & 1) as i64);
        unsafe {
          *self.buffer.get_unchecked_mut(i) = ((cur_fp >> 32) as i64).wrapping_add(res) as u64;
        }
        cur_fp += slope;
        i += 1;
      }
    } else {
      // Slow path: Exceptions
      let mut i = 0;
      let mut ex_idx = ex_offset;
      let mut global_bit_pos = bit_off;
      let mut cur_fp = (start as u128 * slope_fp as u128) as i128 + intercept_fp as i128;
      let slope = slope_fp as i128;

      let mut current_bm_idx = usize::MAX;
      let mut current_bm_word = 0;

      while i < count {
        let g_idx = start + i;
        let bm_idx = g_idx / 64;
        let bm_bit = g_idx % 64;

        if bm_idx != current_bm_idx {
          current_bm_idx = bm_idx;
          current_bm_word = unsafe { *self.pc.bitmap.get_unchecked(bm_idx) };
        }

        let is_ex = (current_bm_word >> bm_bit) & 1 == 1;

        let code = if is_ex {
          let v = unsafe { *self.pc.exceptions.get_unchecked(ex_idx) };
          ex_idx += 1;
          if w > 0 {
            global_bit_pos += w;
          } // Skip residual bits
          v
        } else {
          if w == 0 {
            0
          } else {
            let val = read_bits(&self.pc.residuals, global_bit_pos, w as u8);
            global_bit_pos += w;
            val
          }
        };

        let res = (code >> 1) as i64 ^ -((code & 1) as i64);
        unsafe {
          *self.buffer.get_unchecked_mut(i) = ((cur_fp >> 32) as i64).wrapping_add(res) as u64;
        }
        cur_fp += slope;
        i += 1;
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
}

impl<const B: usize> Iterator for PcRevIterBase<'_, B> {
  type Item = u64;

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    if self.current_idx < 0 {
      return None;
    }
    let idx = self.current_idx as usize;
    // Just use Random Access get_unchecked! It's O(1) now!
    // RevIter is just reverse order random access.
    // Iterating backwards via block buffers is complex.
    // Since we optimized random access to O(1) (2 lookups), calling get_unchecked is quite fast.
    self.current_idx -= 1;
    unsafe { Some(self.pc.get_unchecked(idx)) }
  }
}
