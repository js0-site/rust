use super::{Pc, types::BLOCK_LEN};
use crate::util::bits::read_bits;

pub struct PcIter<'a> {
  pub(crate) pc: &'a Pc,
  /// Current global index
  /// 当前全局索引
  pub(crate) current_idx: usize,
  /// Current segment index
  /// 当前段索引
  pub(crate) current_seg_idx: usize,
  /// Current block index
  /// 当前块索引
  pub(crate) current_block_idx: isize,
  /// Decoding buffer for the current block
  /// 当前块的解码缓冲区
  pub(crate) buffer: [u64; BLOCK_LEN],
  pub(crate) buffer_pos: u16,
  pub(crate) buffer_len: u16,
}

impl Iterator for PcIter<'_> {
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

    // After refill, if we have data, return it
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

impl PcIter<'_> {
  #[inline(never)]
  fn refill_buffer(&mut self) {
    let n = self.pc.len;
    if self.current_idx >= n {
      self.buffer_len = 0;
      return;
    }

    self.current_block_idx += 1;
    let b_idx = self.current_block_idx as usize;
    let meta = unsafe { self.pc.block_meta.get_unchecked(b_idx) };
    let start = b_idx * BLOCK_LEN;
    let end = (start + BLOCK_LEN).min(n);
    let count = end - start;

    let mut i = 0;

    // Prepare bit reading state
    let w = meta.bit_width as usize;
    let mut bit_offset = meta.bit_offset as usize;
    let res = &self.pc.residuals;
    let mask = if w > 0 { !0u64 >> (64 - w) } else { 0 };

    while i < count {
      let global_idx = start + i;
      while self.current_seg_idx + 1 < self.pc.segments.len()
        && global_idx >= self.pc.segments[self.current_seg_idx + 1].start_idx as usize
      {
        self.current_seg_idx += 1;
      }
      let seg = unsafe { self.pc.segments.get_unchecked(self.current_seg_idx) };

      let next_seg_start = if self.current_seg_idx + 1 < self.pc.segments.len() {
        self.pc.segments[self.current_seg_idx + 1].start_idx as usize
      } else {
        n
      };
      // How many items in this segment fall into the current block?
      let sub_count = (next_seg_start - global_idx).min(count - i);

      // Incremental prediction state
      let mut cur_pred_fp =
        (global_idx as u128 * seg.slope_fp as u128) as i128 + seg.intercept_fp as i128;

      if w == 0 {
        // Case: Residuals are all 0
        for j in 0..sub_count {
          let pred = (cur_pred_fp >> 32) as u64;
          unsafe { *self.buffer.get_unchecked_mut(i + j) = pred };
          cur_pred_fp += seg.slope_fp as i128;
        }
      } else {
        // Case: Decode residuals and add to prediction
        for j in 0..sub_count {
          // 1. Decode (Inline ZigZag)
          let word_idx = bit_offset >> 6;
          let bit_idx = (bit_offset & 63) as u8;
          let word = unsafe { *res.get_unchecked(word_idx) };

          let code = if bit_idx + (w as u8) <= 64 {
            (word >> bit_idx) & mask
          } else {
            let word2 = unsafe { *res.get_unchecked(word_idx + 1) };
            let bits1 = 64 - bit_idx;
            let lower = (word >> bit_idx) & (!0u64 >> (64 - bits1));
            let bits2 = (w as u8) - bits1;
            let upper = (word2 & (!0u64 >> (64 - bits2))) << bits1;
            lower | upper
          };

          let residual = (code >> 1) as i64 ^ -((code & 1) as i64);

          // 2. Predict & Combine
          let pred = (cur_pred_fp >> 32) as i64;
          unsafe { *self.buffer.get_unchecked_mut(i + j) = (pred + residual) as u64 };

          // Update state
          cur_pred_fp += seg.slope_fp as i128;
          bit_offset += w;
        }
      }

      i += sub_count;
    }

    self.buffer_pos = 0;
    self.buffer_len = count as u16;
  }
}

pub struct PcRevIter<'a> {
  pub(crate) pc: &'a Pc,
  pub(crate) current_idx: isize,
  pub(crate) current_seg_idx: isize,
  pub(crate) current_block_idx: isize,
}

impl Iterator for PcRevIter<'_> {
  type Item = u64;
  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    if self.current_idx < 0 {
      return None;
    }

    while self.current_seg_idx > 0
      && (self.current_idx as u32) < self.pc.segments[self.current_seg_idx as usize].start_idx
    {
      self.current_seg_idx -= 1;
    }
    let seg = unsafe {
      self
        .pc
        .segments
        .get_unchecked(self.current_seg_idx as usize)
    };

    if self.current_idx < self.current_block_idx * BLOCK_LEN as isize {
      self.current_block_idx -= 1;
    }
    let meta = unsafe {
      self
        .pc
        .block_meta
        .get_unchecked(self.current_block_idx as usize)
    };

    let pred = (((self.current_idx as u128 * seg.slope_fp as u128) as i128
      + seg.intercept_fp as i128)
      >> 32) as i64;
    let code = if meta.bit_width == 0 {
      0
    } else {
      let offset_in_block = self.current_idx as usize % BLOCK_LEN;
      let start_bit = meta.bit_offset as usize + offset_in_block * (meta.bit_width as usize);
      read_bits(&self.pc.residuals, start_bit, meta.bit_width)
    };

    let residual = (code >> 1) as i64 ^ -((code & 1) as i64);
    self.current_idx -= 1;
    Some((pred + residual) as u64)
  }
}
