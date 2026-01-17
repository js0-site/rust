use crate::{
  pgm::{build::build_segments, consts::MIN_EPSILON},
  util::bits::{BitWriter, read_bits},
};

pub mod io;
pub mod iter;
pub mod types;

use self::{
  iter::{PcIterBase, PcRevIterBase},
  types::{BLOCK_LEN, BlockMeta, CompactSegment},
};

/// Type alias using default BLOCK_LEN from build.rs
pub type Pc = PcBase<{ BLOCK_LEN }>;
pub type PcIter<'a> = PcIterBase<'a, { BLOCK_LEN }>;
pub type PcRevIter<'a> = PcRevIterBase<'a, { BLOCK_LEN }>;

/// Compact Learning Index with Blocked Bit-Packing
#[derive(Clone, Debug)]
pub struct PcBase<const B: usize> {
  pub segments: Vec<CompactSegment>,
  pub block_meta: Vec<BlockMeta>,
  pub residuals: Vec<u64>,
  pub exceptions: Vec<u64>,
  pub bitmap: Vec<u64>,
  pub len: usize,
}

impl<const B: usize> PcBase<B> {
  pub fn dump(&self) -> Vec<u8> {
    io::dump(self)
  }
  pub fn load(bytes: &[u8]) -> Self {
    io::load(bytes)
  }

  pub fn new(data: &[u64], epsilon: usize) -> Self {
    let epsilon = epsilon.max(MIN_EPSILON);
    let n = data.len();
    if n == 0 {
      return Self::default();
    }

    let segments = build_segments(data, epsilon);
    let mut compact_segments = Vec::with_capacity(segments.len());
    for s in &segments {
      let slope_inv = if s.slope.abs() < 1e-15 {
        0.0
      } else {
        1.0 / s.slope
      };
      let intercept_shifted = -s.intercept * slope_inv;
      compact_segments.push(CompactSegment {
        start_idx: s.start_idx as u32,
        slope_fp: (slope_inv * (1u64 << 32) as f64) as u64,
        intercept_fp: (intercept_shifted * (1u64 << 32) as f64) as i64,
      });
    }

    let block_count = n.div_ceil(B);
    let mut block_meta = Vec::with_capacity(block_count);
    let mut bit_writer = BitWriter::with_capacity(n);
    let mut bitmap_writer = BitWriter::with_capacity(n / 64 + 1);
    let mut exceptions = Vec::new();
    let mut diff_buffer = Vec::with_capacity(B);

    let mut current_seg_idx = 0;
    for b_idx in 0..block_count {
      let start = b_idx * B;
      let end = (start + B).min(n);
      let block_start_seg_idx = current_seg_idx;
      let mut is_multi_segment = false;

      diff_buffer.clear();
      for (local_idx, &val) in data[start..end].iter().enumerate() {
        let global_idx = (start + local_idx) as u64;
        let prev_seg = current_seg_idx;
        while current_seg_idx + 1 < compact_segments.len()
          && global_idx >= compact_segments[current_seg_idx + 1].start_idx as u64
        {
          current_seg_idx += 1;
        }
        if current_seg_idx != prev_seg {
          is_multi_segment = true;
        }
        let seg = &compact_segments[current_seg_idx];
        let pred =
          ((global_idx as u128 * seg.slope_fp as u128) as i128 + seg.intercept_fp as i128) >> 32;
        let diff = val.wrapping_sub(pred as u64) as i64;
        diff_buffer.push(((diff as u64) << 1) ^ ((diff >> 63) as u64));
      }

      let mut best_w = 0;
      let mut min_cost = u64::MAX;
      let mut counts = [0usize; 65];
      for &d in &diff_buffer {
        counts[if d == 0 {
          0
        } else {
          64 - d.leading_zeros() as usize
        }] += 1;
      }
      let mut num_ex = 0;
      for w in (0..=64).rev() {
        num_ex += if w < 64 { counts[w + 1] } else { 0 };
        let cost = (diff_buffer.len() as u64 * w as u64) + (num_ex as u64 * 64);
        if cost <= min_cost {
          min_cost = cost;
          best_w = w as u8;
        }
      }

      let bit_width = best_w as usize;
      let mask = if bit_width >= 64 {
        !0u64
      } else {
        (1u64 << bit_width) - 1
      };
      let exception_offset = exceptions.len() as u32;
      let mut block_has_ex = false;

      let bit_offset = bit_writer.current_bit_offset() as u32;
      for &code in &diff_buffer {
        if code <= mask {
          bitmap_writer.write(0, 1);
          if bit_width > 0 {
            bit_writer.write(code, bit_width as u8);
          }
        } else {
          bitmap_writer.write(1, 1);
          if bit_width > 0 {
            bit_writer.write(0, bit_width as u8);
          }
          exceptions.push(code);
          block_has_ex = true;
        }
      }

      let mut info = block_start_seg_idx as u32;
      if block_has_ex {
        info |= 0x8000_0000;
      }
      if !is_multi_segment {
        info |= 0x4000_0000;
      }

      block_meta.push(BlockMeta {
        bit_offset,
        bit_width: bit_width as u8,
        seg_idx: info,
        exception_offset,
      });
    }

    Self {
      segments: compact_segments,
      block_meta,
      residuals: bit_writer.finish(),
      exceptions,
      bitmap: bitmap_writer.finish(),
      len: n,
    }
  }

  #[inline(always)]
  pub fn get(&self, index: usize) -> Option<u64> {
    if index >= self.len {
      return None;
    }

    let b_idx = index / B;
    let meta = unsafe { self.block_meta.get_unchecked(b_idx) };
    let info = meta.seg_idx;

    // Step 1: Predict (Fast-path for single segment block)
    let mut s_idx = (info & 0x3FFF_FFFF) as usize;
    if (info & 0x4000_0000) == 0 {
      while s_idx + 1 < self.segments.len()
        && index >= unsafe { self.segments.get_unchecked(s_idx + 1).start_idx as usize }
      {
        s_idx += 1;
      }
    }
    let seg = unsafe { self.segments.get_unchecked(s_idx) };
    let pred =
      (((index as u128 * seg.slope_fp as u128) as i128 + seg.intercept_fp as i128) >> 32) as i64;

    // Step 2: Decode (Fast-path for no exceptions)
    // 步骤 2：解码（无异常值快速路径）
    let w = meta.bit_width as usize;
    let code = if (info & FLAG_HAS_EX) == 0 {
      if w == 0 {
        0
      } else {
        read_bits(
          &self.residuals,
          meta.bit_offset as usize + (index % B) * w,
          w as u8,
        )
      }
    } else {
      let bm_idx = index / 64;
      let bm_bit = index % 64;
      if (unsafe { *self.bitmap.get_unchecked(bm_idx) } >> bm_bit) & 1 == 1 {
        let start_word = (b_idx * B) / 64;
        let mut rank = 0;
        for i in start_word..bm_idx {
          rank += unsafe { self.bitmap.get_unchecked(i).count_ones() };
        }
        rank +=
          (unsafe { *self.bitmap.get_unchecked(bm_idx) } & ((1u64 << bm_bit) - 1)).count_ones();
        unsafe {
          *self
            .exceptions
            .get_unchecked(meta.exception_offset as usize + rank as usize)
        }
      } else {
        if w == 0 {
          0
        } else {
          read_bits(
            &self.residuals,
            meta.bit_offset as usize + (index % B) * w,
            w as u8,
          )
        }
      }
    };

    let res = (code >> 1) as i64 ^ -((code & 1) as i64);
    Some(pred.wrapping_add(res) as u64)
  }

  pub fn iter(&self) -> PcIterBase<'_, B> {
    PcIterBase {
      pc: self,
      current_idx: 0,
      current_seg_idx: 0,
      current_block_idx: -1,
      buffer: vec![0; B],
      buffer_pos: 0,
      buffer_len: 0,
    }
  }

  /// Create reverse iterator
  /// 创建逆向迭代器
  pub fn rev_iter(&self) -> PcRevIterBase<'_, B> {
    PcRevIterBase {
      pc: self,
      current_idx: self.len as isize - 1,
      current_seg_idx: self.segments.len() as isize - 1,
    }
  }

  pub fn size_in_bytes(&self) -> usize {
    std::mem::size_of::<Self>()
      + self.segments.len() * 20
      + self.block_meta.len() * 13
      + self.residuals.len() * 8
      + self.exceptions.len() * 8
      + self.bitmap.len() * 8
  }
}

impl<const B: usize> Default for PcBase<B> {
  fn default() -> Self {
    Self {
      segments: vec![],
      block_meta: vec![],
      residuals: vec![],
      exceptions: vec![],
      bitmap: vec![],
      len: 0,
    }
  }
}
