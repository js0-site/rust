use std::mem;

use crate::{
  error::Result,
  util::bits::{BitWriter, read_bits},
};

pub mod io;
pub mod iter;
pub mod types;

use self::{
  iter::{PcIterBase, PcRevIterBase},
  types::{BLOCK_LEN, BlockMeta, FLAG_HAS_EX, PcConf},
};

/// Type alias using default BLOCK_LEN from build.rs.
/// 使用 build.rs 生成的默认 BLOCK_LEN 的类型别名。
pub type Pc = PcBase<{ BLOCK_LEN }>;
// Iterators disabled temporarily during refactor or need update
pub type PcIter<'a> = PcIterBase<'a, { BLOCK_LEN }>;
pub type PcRevIter<'a> = PcRevIterBase<'a, { BLOCK_LEN }>;

/// Compact Learning Index with Blocked Bit-Packing (Block-Local Prediction).
/// 紧凑的学习型索引，使用分块位压缩优化空间（块级本地预测）。
#[derive(Clone, Debug)]
pub struct PcBase<const B: usize> {
  // segments removed
  pub block_meta: Vec<BlockMeta>,
  pub residuals: Vec<u64>,
  pub exceptions: Vec<u64>,
  pub bitmap: Vec<u64>,
  pub len: usize,
}

impl<const B: usize> PcBase<B> {
  /// Serialize to bytes.
  /// 序列化为字节流。
  pub fn dump(&self) -> Vec<u8> {
    io::dump(self)
  }

  /// Deserialize from bytes.
  /// 从字节流反序列化。
  pub fn load(bytes: &[u8]) -> Result<Self> {
    io::load(bytes)
  }

  /// Build with block-local prediction.
  pub fn new(data: &[u64], epsilon: usize) -> Self {
    Self::new_with_conf(data, PcConf::with_epsilon(epsilon))
  }

  /// Build with custom configuration.
  pub fn new_with_conf(data: &[u64], conf: PcConf) -> Self {
    // epsilon is ignored for segmentation, but could be used implicitly?
    // In this Block-Local model, we don't satisfy epsilon. We maximize bit-packing.
    let ex_penalty = conf.ex_penalty.get() as u64;
    let n = data.len();
    if n == 0 {
      return Self::default();
    }

    let block_count = n.div_ceil(B);
    let mut block_meta = Vec::with_capacity(block_count);
    let mut bit_writer = BitWriter::with_capacity(n);
    let mut bitmap_writer = BitWriter::with_capacity(n / 64 + 1);
    let mut exceptions = Vec::new();
    let mut diff_buffer = Vec::with_capacity(B);

    for b_idx in 0..block_count {
      let start = b_idx * B;
      let end = (start + B).min(n);
      let block_data = &data[start..end];

      // 1. Calculate Block Model (First-Last Line)
      // y = slope * x + intercept
      let first_val = block_data[0];
      let last_val = block_data[block_data.len() - 1];
      let start_x = start as f64;
      let end_x = (end - 1) as f64;

      let slope = if end > start + 1 {
        (last_val as f64 - first_val as f64) / (end_x - start_x)
      } else {
        0.0
      };
      let intercept = first_val as f64 - slope * start_x;

      let slope_fp = (slope * (1u64 << 32) as f64) as u64;
      let intercept_fp = (intercept * (1u64 << 32) as f64) as i64;

      // 2. Compute Residuals
      diff_buffer.clear();
      for (local_idx, &val) in block_data.iter().enumerate() {
        let global_idx = (start + local_idx) as u64;
        let pred = ((global_idx as u128 * slope_fp as u128) as i128 + intercept_fp as i128) >> 32;
        let diff = val.wrapping_sub(pred as u64) as i64;
        // ZigZag encode
        diff_buffer.push(((diff as u64) << 1) ^ ((diff >> 63) as u64));
      }

      // 3. Find optimal bit width
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
        let cost = (diff_buffer.len() as u64 * w as u64) + (num_ex as u64 * 64 * ex_penalty);
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

      let flags = if block_has_ex { FLAG_HAS_EX } else { 0 };

      block_meta.push(BlockMeta {
        bit_offset,
        bit_width: bit_width as u8,
        flags,
        exception_offset,
        slope_fp,
        intercept_fp,
      });
    }

    Self {
      block_meta,
      residuals: bit_writer.finish(),
      exceptions,
      bitmap: bitmap_writer.finish(),
      len: n,
    }
  }

  /// Get value at index.
  #[inline(always)]
  pub fn get(&self, index: usize) -> Option<u64> {
    if index >= self.len {
      return None;
    }
    unsafe { Some(self.get_unchecked(index)) }
  }

  /// Get value at index without bounds check.
  /// # Safety
  /// Caller must ensure index < self.len().
  #[inline(always)]
  pub unsafe fn get_unchecked(&self, index: usize) -> u64 {
    let b_idx = index / B;
    // Step 1: Lookup Meta (1 cache miss)
    // SAFETY: index checked by caller. b_idx valid.
    let meta = unsafe { self.block_meta.get_unchecked(b_idx) };

    // Step 2: Predict using Block Model (Inline)
    let pred = ((index as u128 * meta.slope_fp as u128) as i128 + meta.intercept_fp as i128) >> 32;

    // Step 3: Decode Residual (1 cache miss if no flags, 2 if exception)
    let w = meta.bit_width as usize;
    if (meta.flags & FLAG_HAS_EX) == 0 {
      // Hot path: No exceptions
      let code = if w == 0 {
        0
      } else {
        read_bits(
          &self.residuals,
          meta.bit_offset as usize + (index % B) * w,
          w as u8,
        )
      };

      let res = (code >> 1) as i64 ^ -((code & 1) as i64);
      (pred as u64).wrapping_add(res as u64)
    } else {
      unsafe { self.get_exception_cold(index, b_idx, meta, pred) }
    }
  }

  #[cold]
  unsafe fn get_exception_cold(
    &self,
    index: usize,
    b_idx: usize,
    meta: &BlockMeta,
    pred: i128,
  ) -> u64 {
    let bm_idx = index / 64;
    let bm_bit = index % 64;

    // Check if current index is an exception
    if (unsafe { *self.bitmap.get_unchecked(bm_idx) } >> bm_bit) & 1 == 1 {
      let start_word = (b_idx * B) / 64;
      let mut rank = 0;
      // Count ones in previous words within the block range
      // Since B is small, this loop runs 0-2 times usually.
      for i in start_word..bm_idx {
        rank += unsafe { self.bitmap.get_unchecked(i).count_ones() };
      }
      // Count ones in current word up to current bit
      rank += (unsafe { *self.bitmap.get_unchecked(bm_idx) } & ((1u64 << bm_bit) - 1)).count_ones();
      unsafe {
        *self
          .exceptions
          .get_unchecked(meta.exception_offset as usize + rank as usize)
      }
    } else {
      // It's a normal value in a block that has SOME exceptions
      let w = meta.bit_width as usize;
      let code = if w == 0 {
        0
      } else {
        read_bits(
          &self.residuals,
          meta.bit_offset as usize + (index % B) * w,
          w as u8,
        )
      };
      let res = (code >> 1) as i64 ^ -((code & 1) as i64);
      (pred as u64).wrapping_add(res as u64)
    }
  }

  pub fn iter(&self) -> PcIterBase<'_, B> {
    PcIterBase {
      pc: self,
      current_idx: 0,
      current_block_idx: -1,
      buffer: vec![0; B],
      buffer_pos: 0,
      buffer_len: 0,
    }
  }

  /// Create reverse iterator.
  /// 创建逆向迭代器。
  pub fn rev_iter(&self) -> PcRevIterBase<'_, B> {
    PcRevIterBase {
      pc: self,
      current_idx: self.len as isize - 1,
    }
  }

  pub fn print_stats(&self) {
    let mut total_blocks = 0;
    let mut w_counts = [0usize; 65];
    let mut has_ex_blocks = 0;

    for meta in &self.block_meta {
      total_blocks += 1;
      w_counts[meta.bit_width as usize] += 1;
      if (meta.flags & FLAG_HAS_EX) != 0 {
        has_ex_blocks += 1;
      }
    }

    // Count exceptions
    let total_exceptions = self.exceptions.len();

    println!("--- PC Stats (Block-Local) ---");
    println!("Total Keys: {}", self.len);
    println!("Total Blocks: {}", total_blocks);
    println!(
      "Exceptions: {} ({:.2}%)",
      total_exceptions,
      total_exceptions as f64 / self.len as f64 * 100.0
    );
    println!(
      "Blocks with Exceptions: {} ({:.2}%)",
      has_ex_blocks,
      has_ex_blocks as f64 / total_blocks as f64 * 100.0
    );
    println!("Width (w) Distribution (Blocks):");
    for (w, count) in w_counts.iter().enumerate() {
      if *count > 0 {
        println!(
          "  w={}: {} ({:.2}%)",
          w,
          count,
          *count as f64 / total_blocks as f64 * 100.0
        );
      }
    }
  }

  pub fn size_in_bytes(&self) -> usize {
    mem::size_of::<Self>()
      + self.block_meta.len() * mem::size_of::<BlockMeta>()
      + self.residuals.len() * 8
      + self.exceptions.len() * 8
      + self.bitmap.len() * 8
  }
}

impl<const B: usize> Default for PcBase<B> {
  fn default() -> Self {
    Self {
      block_meta: vec![],
      residuals: vec![],
      exceptions: vec![],
      bitmap: vec![],
      len: 0,
    }
  }
}
