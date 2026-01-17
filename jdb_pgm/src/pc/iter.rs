//! Iterator implementations for Pc
//! Pc 迭代器实现

use super::PcBase;
use crate::util::bits::read_bits;

/// Forward iterator for Pc
/// Pc 正向迭代器
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
  /// Refill buffer with next block's decoded values
  /// 用下一个块的解码值填充缓冲区
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
    let mut bit_off = meta.bit_offset as usize;
    let mut ex_idx = meta.exception_offset as usize;

    // Check if this block has exceptions via flag bit
    // 通过标志位判断是否有异常值
    let has_ex = (meta.seg_idx & 0x8000_0000) != 0;

    let mut bm_word = 0u64;
    let mut bm_bit = 64u8;

    let mut i = 0;
    while i < count {
      let g_idx = start + i;
      // Advance segment if needed
      // 必要时推进到下一个段
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
      for j in 0..sub_count {
        let code = if has_ex {
          if bm_bit >= 64 {
            bm_word = unsafe { *self.pc.bitmap.get_unchecked((g_idx + j) / 64) };
            bm_bit = ((g_idx + j) % 64) as u8;
          }
          let is_ex = (bm_word >> bm_bit) & 1 == 1;
          bm_bit += 1;
          if is_ex {
            let v = unsafe { *self.pc.exceptions.get_unchecked(ex_idx) };
            ex_idx += 1;
            if w > 0 {
              bit_off += w;
            }
            v
          } else if w == 0 {
            0
          } else {
            let val = read_bits(&self.pc.residuals, bit_off, w as u8);
            bit_off += w;
            val
          }
        } else if w == 0 {
          0
        } else {
          let v = read_bits(&self.pc.residuals, bit_off, w as u8);
          bit_off += w;
          v
        };
        let res = (code >> 1) as i64 ^ -((code & 1) as i64);
        unsafe {
          *self.buffer.get_unchecked_mut(i + j) = ((cur_fp >> 32) as i64 + res) as u64;
        }
        cur_fp += seg.slope_fp as i128;
      }
      i += sub_count;
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
    let has_ex = (meta.seg_idx & 0x8000_0000) != 0;

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
