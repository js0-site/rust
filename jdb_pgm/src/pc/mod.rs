use crate::{build::build_segments, consts::MIN_EPSILON};

mod patch;
use patch::{BitWriter, read_bits};

/// Compact Learning Index with Blocked Bit-Packing
/// 紧凑的学习型索引，使用分块位压缩优化空间
#[derive(Clone, Debug)]
pub struct Pc {
  pub segments: Vec<CompactSegment>,
  pub block_meta: Vec<BlockMeta>,
  pub residuals: Vec<u64>,
  pub len: usize,
}

#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct CompactSegment {
  pub start_idx: u32,
  /// Fixed-point: Key = (index * slope_fp + intercept_fp) >> 32
  pub slope_fp: u64,
  pub intercept_fp: i64,
}

#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct BlockMeta {
  pub bit_offset: u32,
  pub bit_width: u8,
}

const BLOCK_LEN: usize = 128;

impl Pc {
  /// Serialize to bytes
  /// 序列化为字节流
  pub fn dump(&self) -> Vec<u8> {
    let mut out = Vec::with_capacity(self.size_in_bytes());
    out.extend_from_slice(&(self.len as u64).to_le_bytes());

    // segments
    out.extend_from_slice(&(self.segments.len() as u32).to_le_bytes());
    for s in &self.segments {
      out.extend_from_slice(&s.start_idx.to_le_bytes());
      out.extend_from_slice(&s.slope_fp.to_le_bytes());
      out.extend_from_slice(&s.intercept_fp.to_le_bytes());
    }

    // block_meta
    out.extend_from_slice(&(self.block_meta.len() as u32).to_le_bytes());
    for b in &self.block_meta {
      out.extend_from_slice(&b.bit_offset.to_le_bytes());
      out.push(b.bit_width);
    }

    // residuals
    out.extend_from_slice(&(self.residuals.len() as u32).to_le_bytes());
    for r in &self.residuals {
      out.extend_from_slice(&r.to_le_bytes());
    }
    out
  }

  /// Deserialize from bytes
  /// 从字节流反序列化
  pub fn load(bytes: &[u8]) -> Self {
    let mut pos = 0;

    let len = u64::from_le_bytes(bytes[pos..pos + 8].try_into().unwrap()) as usize;
    pos += 8;

    // segments
    let seg_count = u32::from_le_bytes(bytes[pos..pos + 4].try_into().unwrap()) as usize;
    pos += 4;
    let mut segments = Vec::with_capacity(seg_count);
    for _ in 0..seg_count {
      let start_idx = u32::from_le_bytes(bytes[pos..pos + 4].try_into().unwrap());
      pos += 4;
      let slope_fp = u64::from_le_bytes(bytes[pos..pos + 8].try_into().unwrap());
      pos += 8;
      let intercept_fp = i64::from_le_bytes(bytes[pos..pos + 8].try_into().unwrap());
      pos += 8;
      segments.push(CompactSegment {
        start_idx,
        slope_fp,
        intercept_fp,
      });
    }

    // block_meta
    let block_count = u32::from_le_bytes(bytes[pos..pos + 4].try_into().unwrap()) as usize;
    pos += 4;
    let mut block_meta = Vec::with_capacity(block_count);
    for _ in 0..block_count {
      let bit_offset = u32::from_le_bytes(bytes[pos..pos + 4].try_into().unwrap());
      pos += 4;
      let bit_width = bytes[pos];
      pos += 1;
      block_meta.push(BlockMeta {
        bit_offset,
        bit_width,
      });
    }

    // residuals
    let res_count = u32::from_le_bytes(bytes[pos..pos + 4].try_into().unwrap()) as usize;
    pos += 4;
    let mut residuals = Vec::with_capacity(res_count);
    for _ in 0..res_count {
      residuals.push(u64::from_le_bytes(bytes[pos..pos + 8].try_into().unwrap()));
      pos += 8;
    }

    Self {
      segments,
      block_meta,
      residuals,
      len,
    }
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
        let pred = ((global_idx * seg.slope_fp) as i64 + seg.intercept_fp) >> 32;
        let diff = val as i64 - pred;

        let encoded = if diff >= 0 {
          (diff as u64) << 1
        } else {
          ((-diff) as u64) << 1 | 1
        };
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
    let pred = ((index as u64 * seg.slope_fp) as i64 + seg.intercept_fp) >> 32;

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
      current_block_idx: 0,
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

pub struct PcIter<'a> {
  pc: &'a Pc,
  current_idx: usize,
  current_seg_idx: usize,
  current_block_idx: usize,
}

impl Iterator for PcIter<'_> {
  type Item = u64;
  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    if self.current_idx >= self.pc.len {
      return None;
    }

    while self.current_seg_idx + 1 < self.pc.segments.len()
      && self.current_idx >= self.pc.segments[self.current_seg_idx + 1].start_idx as usize
    {
      self.current_seg_idx += 1;
    }
    let seg = unsafe { self.pc.segments.get_unchecked(self.current_seg_idx) };

    if self.current_idx >= (self.current_block_idx + 1) * BLOCK_LEN {
      self.current_block_idx += 1;
    }
    let meta = unsafe { self.pc.block_meta.get_unchecked(self.current_block_idx) };

    let pred = ((self.current_idx as u64 * seg.slope_fp) as i64 + seg.intercept_fp) >> 32;
    let res = &self.pc.residuals;
    let code = if meta.bit_width == 0 {
      0
    } else {
      let offset_in_block = self.current_idx % BLOCK_LEN;
      let start_bit = meta.bit_offset as usize + offset_in_block * (meta.bit_width as usize);

      let word_idx = start_bit >> 6;
      let bit_idx = (start_bit & 63) as u8;

      if bit_idx + meta.bit_width <= 64 {
        let word = unsafe { *res.get_unchecked(word_idx) };
        (word >> bit_idx) & (!0u64 >> (64 - meta.bit_width))
      } else {
        let word1 = unsafe { *res.get_unchecked(word_idx) };
        let word2 = unsafe { *res.get_unchecked(word_idx + 1) };
        let bits1 = 64 - bit_idx;
        let lower = (word1 >> bit_idx) & (!0u64 >> (64 - bits1));
        let bits2 = meta.bit_width - bits1;
        let upper = (word2 & (!0u64 >> (64 - bits2))) << bits1;
        lower | upper
      }
    };

    let residual = (code >> 1) as i64 ^ -((code & 1) as i64);
    self.current_idx += 1;
    Some((pred + residual) as u64)
  }
}

pub struct PcRevIter<'a> {
  pc: &'a Pc,
  current_idx: isize,
  current_seg_idx: isize,
  current_block_idx: isize,
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

    let pred = ((self.current_idx as u64 * seg.slope_fp) as i64 + seg.intercept_fp) >> 32;
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
