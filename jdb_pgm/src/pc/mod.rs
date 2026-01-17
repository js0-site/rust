use crate::{
  pgm::{build::build_segments, consts::MIN_EPSILON},
  util::bits::{BitWriter, read_bits},
};

pub mod io;
pub mod iter;
pub mod types;

use self::{
  iter::{PcIter, PcRevIter},
  types::{BLOCK_LEN, BlockMeta, CompactSegment},
};

/// Compact Learning Index with Blocked Bit-Packing
/// 紧凑的学习型索引，使用分块位压缩优化空间
#[derive(Clone, Debug)]
pub struct Pc {
  pub segments: Vec<CompactSegment>,
  pub block_meta: Vec<BlockMeta>,
  pub residuals: Vec<u64>,
  pub len: usize,
}

impl Pc {
  /// Serialize to bytes
  /// 序列化为字节流
  pub fn dump(&self) -> Vec<u8> {
    io::dump(self)
  }

  /// Deserialize from bytes
  /// 从字节流反序列化
  pub fn load(bytes: &[u8]) -> Self {
    io::load(bytes)
  }

  /// Build a compact PGM index with blocked bit-packing
  /// 构建紧凑的分块 PGM 索引
  pub fn new(data: &[u64], epsilon: usize) -> Self {
    let epsilon = epsilon.max(MIN_EPSILON);
    let n = data.len();

    if n == 0 {
      return Self {
        segments: vec![],
        block_meta: vec![],
        residuals: vec![],
        len: 0,
      };
    }

    // 1. Build original PGM segments (Value -> Rank)
    let segments = build_segments(data, epsilon);
    let mut compact_segments = Vec::with_capacity(segments.len());

    for s in &segments {
      // Key = (index - intercept) * slope_inv
      let slope_inv = if s.slope.abs() < 1e-15 {
        0.0
      } else {
        1.0 / s.slope
      };
      let intercept_shifted = -s.intercept * slope_inv;

      // Convert to fixed-point (32.32)
      let slope_fp = (slope_inv * (1u64 << 32) as f64) as u64;
      let intercept_fp = (intercept_shifted * (1u64 << 32) as f64) as i64;

      compact_segments.push(CompactSegment {
        start_idx: s.start_idx as u32,
        slope_fp,
        intercept_fp,
      });
    }

    // 2. Blocked compression (Residuals = Key - Prediction)
    let block_count = n.div_ceil(BLOCK_LEN);
    let mut block_meta = Vec::with_capacity(block_count);
    let mut bit_writer = BitWriter::with_capacity(n * 4);

    let mut current_seg_idx = 0;
    for b_idx in 0..block_count {
      let start = b_idx * BLOCK_LEN;
      let end = (start + BLOCK_LEN).min(n);
      let chunk = &data[start..end];

      let mut encoded_buffer = Vec::with_capacity(BLOCK_LEN);
      let mut max_diff = 0u64;

      for (local_idx, &val) in chunk.iter().enumerate() {
        let global_idx = (start + local_idx) as u64;
        while current_seg_idx + 1 < compact_segments.len()
          && global_idx >= compact_segments[current_seg_idx + 1].start_idx as u64
        {
          current_seg_idx += 1;
        }
        let seg = &compact_segments[current_seg_idx];

        // Prediction using fixed-point
        // Use u128 to prevent overflow during multiplication: (index * slope) can be > u64::MAX
        let pred =
          ((global_idx as u128 * seg.slope_fp as u128) as i128 + seg.intercept_fp as i128) >> 32;

        let diff = val.wrapping_sub(pred as u64) as i64;
        let encoded = ((diff as u64) << 1) ^ ((diff >> 63) as u64);

        if encoded > max_diff {
          max_diff = encoded;
        }
        encoded_buffer.push(encoded);
      }

      let bit_width = if max_diff == 0 {
        0
      } else {
        64 - max_diff.leading_zeros() as u8
      };
      let bit_offset = bit_writer.current_bit_offset() as u32;

      block_meta.push(BlockMeta {
        bit_offset,
        bit_width,
      });
      if bit_width > 0 {
        for &code in &encoded_buffer {
          bit_writer.write(code, bit_width);
        }
      }
    }

    Self {
      segments: compact_segments,
      block_meta,
      residuals: bit_writer.finish(),
      len: n,
    }
  }

  #[inline(always)]
  pub fn get(&self, index: usize) -> Option<u64> {
    if index >= self.len {
      return None;
    }

    let seg_idx = self
      .segments
      .partition_point(|s| s.start_idx as usize <= index)
      - 1;
    let seg = unsafe { self.segments.get_unchecked(seg_idx) };
    let pred =
      (((index as u128 * seg.slope_fp as u128) as i128 + seg.intercept_fp as i128) >> 32) as i64;

    let b_idx = index / BLOCK_LEN;
    let meta = unsafe { self.block_meta.get_unchecked(b_idx) };
    if meta.bit_width == 0 {
      return Some(pred as u64);
    }

    let offset_in_block = index % BLOCK_LEN;
    let start_bit = meta.bit_offset as usize + offset_in_block * (meta.bit_width as usize);
    let code = read_bits(&self.residuals, start_bit, meta.bit_width);

    let residual = (code >> 1) as i64 ^ -((code & 1) as i64);
    Some((pred + residual) as u64)
  }

  pub fn iter(&self) -> PcIter<'_> {
    PcIter {
      pc: self,
      current_idx: 0,
      current_seg_idx: 0,
      current_block_idx: -1,
      buffer: [0; BLOCK_LEN],
      buffer_pos: 0,
      buffer_len: 0,
    }
  }

  pub fn rev_iter(&self) -> PcRevIter<'_> {
    PcRevIter {
      pc: self,
      current_idx: self.len as isize - 1,
      current_seg_idx: self.segments.len() as isize - 1,
      current_block_idx: (self.len as isize - 1) / BLOCK_LEN as isize,
    }
  }

  pub fn size_in_bytes(&self) -> usize {
    std::mem::size_of::<Self>()
      + self.segments.len() * std::mem::size_of::<CompactSegment>()
      + self.block_meta.len() * 5
      + self.residuals.len() * 8
  }
}
