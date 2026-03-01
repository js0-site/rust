//! Compact Bit Reader/Writer for PGM residuals
//! 用于 PGM 残差的紧凑位读写器

#[inline(always)]
pub fn read_bits(data: &[u64], start_bit: usize, bits: u8) -> u64 {
  let word_idx = start_bit / 64;
  let bit_idx = (start_bit % 64) as u8;

  if bit_idx + bits <= 64 {
    let word = unsafe { *data.get_unchecked(word_idx) };
    (word >> bit_idx) & (!0u64 >> (64 - bits))
  } else {
    let word1 = unsafe { *data.get_unchecked(word_idx) };
    // SAFETY: BitWriter guarantees that if a value spans two words,
    // the second word is also pushed (even if partial).
    let word2 = unsafe { *data.get_unchecked(word_idx + 1) };
    let bits1 = 64 - bit_idx;
    let lower = (word1 >> bit_idx) & (!0u64 >> (64 - bits1));
    let bits2 = bits - bits1;
    let upper = (word2 & (!0u64 >> (64 - bits2))) << bits1;
    lower | upper
  }
}

pub struct BitWriter {
  pub data: Vec<u64>,
  current: u64,
  bits_in_current: u8,
  total_bits: usize,
}

impl BitWriter {
  pub fn with_capacity(bits: usize) -> Self {
    Self {
      data: Vec::with_capacity(bits.div_ceil(64)),
      current: 0,
      bits_in_current: 0,
      total_bits: 0,
    }
  }

  pub fn write(&mut self, val: u64, bits: u8) {
    if bits == 0 {
      return;
    }
    let val = val & (!0u64 >> (64 - bits));

    if self.bits_in_current + bits <= 64 {
      self.current |= val << self.bits_in_current;
      self.bits_in_current += bits;
      if self.bits_in_current == 64 {
        self.data.push(self.current);
        self.current = 0;
        self.bits_in_current = 0;
      }
    } else {
      let first_part = 64 - self.bits_in_current;
      self.current |= val << self.bits_in_current;
      self.data.push(self.current);
      self.bits_in_current = bits - first_part;
      self.current = val >> first_part;
    }
    self.total_bits += bits as usize;
  }

  pub fn current_bit_offset(&self) -> usize {
    self.total_bits
  }

  pub fn finish(mut self) -> Vec<u64> {
    if self.bits_in_current > 0 {
      self.data.push(self.current);
    }
    // Padding for unaligned 64-bit reads
    self.data.push(0);
    self.data
  }
}
