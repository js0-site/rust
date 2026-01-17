use bitpacking::{BitPacker, BitPacker4x};

use super::Patch;
use crate::Segment;

#[derive(Clone, Debug)]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
pub struct BitPackedPatch {
  // Stores the bit-packed residuals.
  // Generally using BitPacker4x (128 elements per block).
  pub packet_data: Vec<u8>,
  pub num_bits_per_block: Vec<u8>,
  pub original_len: usize,
}

impl Patch<u64> for BitPackedPatch {
  fn on_segments_built(data: &[u64], segments: &[Segment<u64>]) -> Self {
    if data.is_empty() {
      return Self {
        packet_data: vec![],
        num_bits_per_block: vec![],
        original_len: 0,
      };
    }

    let bitpacker = BitPacker4x::new();
    let block_len = BitPacker4x::BLOCK_LEN;
    let mut residuals = Vec::with_capacity(data.len());

    let mut seg_idx = 0;
    let mut seg = segments[0];

    for (i, &val) in data.iter().enumerate() {
      // Advance segment if needed
      while i >= seg.end_idx && seg_idx + 1 < segments.len() {
        seg_idx += 1;
        seg = segments[seg_idx];
      }

      let pred_rank = i as f64;
      // x_pred = (rank - intercept) / slope
      let pred_key_f64 = if seg.slope.abs() < 1e-9 {
        0.0
      } else {
        (pred_rank - seg.intercept) / seg.slope
      };

      let pred_key = pred_key_f64 as i64;
      let real_key = val as i64;
      let diff = real_key - pred_key;

      // ZigZag Encode
      let zig_zag = if diff >= 0 {
        (diff as u64) * 2
      } else {
        (-diff as u64) * 2 - 1
      };

      residuals.push(zig_zag as u32);
    }

    // BitPacking (Padding to 128)
    let padding_needed = (block_len - (residuals.len() % block_len)) % block_len;
    residuals.extend(std::iter::repeat_n(0, padding_needed));

    let mut packet_data = Vec::new();
    let mut num_bits_per_block = Vec::new();

    for chunk in residuals.chunks(block_len) {
      let num_bits = bitpacker.num_bits(chunk);
      let mut packed = vec![0u8; (block_len * num_bits as usize) / 8 + 8];
      let _len = bitpacker.compress(chunk, &mut packed[..], num_bits);
      packet_data.extend_from_slice(&packed[0..(block_len * num_bits as usize).div_ceil(8)]);
      num_bits_per_block.push(num_bits);
    }

    Self {
      packet_data,
      num_bits_per_block,
      original_len: data.len(),
    }
  }
}

impl BitPackedPatch {
  #[inline]
  pub fn get_residual(&self, index: usize) -> u32 {
    if index >= self.original_len {
      return 0; // Or panic, but returning 0 is safe-ish for out of bounds if we assume caller checks
    }

    let block_len = BitPacker4x::BLOCK_LEN;
    let block_idx = index / block_len;
    let offset_in_block = index % block_len;

    let num_bits = self.num_bits_per_block[block_idx] as usize;

    // Finding byte start:
    // Ideally we store offsets, but here we just sum.

    let bitpacker = BitPacker4x::new();
    let mut decompressed = [0u32; BitPacker4x::BLOCK_LEN];

    // Decompress the block
    let _slice_end = if block_idx == self.num_bits_per_block.len() - 1 {
      self.packet_data.len()
    } else {
      // This logic is tricky without explicit offsets.
      // Let's iterate to find offset.
      let mut offset = 0;
      for i in 0..block_idx {
        let bits = self.num_bits_per_block[i] as usize;
        offset += (bits * block_len).div_ceil(8);
      }
      let current_bits = self.num_bits_per_block[block_idx] as usize;
      offset + (current_bits * block_len).div_ceil(8)
    };

    let slice_start = {
      let mut offset = 0;
      for i in 0..block_idx {
        let bits = self.num_bits_per_block[i] as usize;
        offset += (bits * block_len).div_ceil(8);
      }
      offset
    };

    // Access packet_data
    let _ = bitpacker.decompress(
      &self.packet_data[slice_start..],
      &mut decompressed,
      num_bits as u8,
    );

    decompressed[offset_in_block]
  }

  // Optimize: Iterator that keeps state
  pub fn iter(&self) -> BitPackedPatchIter<'_> {
    BitPackedPatchIter::new(self)
  }

  pub fn rev_iter(&self) -> BitPackedPatchRevIter<'_> {
    BitPackedPatchRevIter::new(self)
  }
}

pub struct BitPackedPatchIter<'a> {
  patch: &'a BitPackedPatch,
  block_idx: usize,
  in_block_idx: usize,
  // precise byte offset
  byte_offset: usize,
  current_block: [u32; BitPacker4x::BLOCK_LEN],
  current_count: usize, // total items yielded
}

impl<'a> BitPackedPatchIter<'a> {
  fn new(patch: &'a BitPackedPatch) -> Self {
    Self {
      patch,
      block_idx: 0,
      in_block_idx: BitPacker4x::BLOCK_LEN, // Force load first block
      byte_offset: 0,
      current_block: [0u32; BitPacker4x::BLOCK_LEN],
      current_count: 0,
    }
  }
}

impl<'a> Iterator for BitPackedPatchIter<'a> {
  type Item = u32;

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    if self.current_count >= self.patch.original_len {
      return None;
    }

    if self.in_block_idx >= BitPacker4x::BLOCK_LEN {
      // Load next block
      if self.block_idx >= self.patch.num_bits_per_block.len() {
        return None;
      }

      let num_bits = self.patch.num_bits_per_block[self.block_idx] as usize;
      let bitpacker = BitPacker4x::new();

      // Decompress from current byte_offset
      let _ = bitpacker.decompress(
        &self.patch.packet_data[self.byte_offset..],
        &mut self.current_block,
        num_bits as u8,
      );

      // Advance state
      self.byte_offset += (num_bits * BitPacker4x::BLOCK_LEN).div_ceil(8);
      self.block_idx += 1;
      self.in_block_idx = 0;
    }

    let val = self.current_block[self.in_block_idx];
    self.in_block_idx += 1;
    self.current_count += 1;
    Some(val)
  }
}

pub struct BitPackedPatchRevIter<'a> {
  patch: &'a BitPackedPatch,
  block_idx: isize,          // Current block index (going down)
  in_block_idx: isize,       // Current item index inside block (going down)
  block_offsets: Vec<usize>, // Precomputed offsets for each block
  current_block: [u32; BitPacker4x::BLOCK_LEN],
  current_count: usize,
}

impl<'a> BitPackedPatchRevIter<'a> {
  fn new(patch: &'a BitPackedPatch) -> Self {
    // Precompute offsets
    let mut block_offsets = Vec::with_capacity(patch.num_bits_per_block.len());
    let mut offset = 0;
    for &bits in &patch.num_bits_per_block {
      block_offsets.push(offset);
      offset += (bits as usize * BitPacker4x::BLOCK_LEN).div_ceil(8);
    }

    let total_items = patch.original_len;
    if total_items == 0 {
      return Self {
        patch,
        block_idx: -1,
        in_block_idx: -1,
        block_offsets: vec![],
        current_block: [0u32; BitPacker4x::BLOCK_LEN],
        current_count: 0,
      };
    }

    let last_block_idx = (total_items - 1) / BitPacker4x::BLOCK_LEN;
    let _last_item_in_block = (total_items - 1) % BitPacker4x::BLOCK_LEN;

    Self {
      patch,
      block_idx: last_block_idx as isize,
      in_block_idx: -(BitPacker4x::BLOCK_LEN as isize), // Force load
      block_offsets,
      current_block: [0u32; BitPacker4x::BLOCK_LEN],
      current_count: 0,
    }
    // Actually, we can just load the last block immediately or lazy load.
    // Let's lazy load but set indices such that next() triggers load.
    // Wait, lazy load logic:
    // if in_block_idx < 0 { load block_idx; in_block_idx = max }
    // For first call: block_idx = last; in_block_idx = last_item_in_block (NOT max).
    // Exceptions: the last block (which is first processed) might be partial?
    // No, construction pads to 128 (BLOCK_LEN). So it handles it fine.
    // BUT `original_len` implies we should stop.
    // Actually, iterating residuals:
    // PGM logic: we need residual for item N-1, N-2...
    // `residuals` vector was padded. `get_residual` retrieves them.
    // So we can assume full blocks, but valid items are only `original_len`.
    // My `iter` implementation checks `current_count`.
    // `rev_iter` should also check count.
  }
}

impl<'a> Iterator for BitPackedPatchRevIter<'a> {
  type Item = u32;

  fn next(&mut self) -> Option<Self::Item> {
    if self.current_count >= self.patch.original_len {
      return None;
    }

    // Logic adjusted:
    // We start pointing at (last_block, last_item_in_slot).
    // But `new` initialization is static.
    // Let's make `new` load the block immediately or set state carefully.

    // Re-init logic in `new` was:
    // block_idx = last_block
    // in_block_idx = -1 (Force load?)
    // Let's clean up.

    // If we haven't loaded the current block (sentinel)
    if self.in_block_idx < 0 {
      if self.block_idx < 0 {
        return None;
      }

      let b_idx = self.block_idx as usize;
      let num_bits = self.patch.num_bits_per_block[b_idx] as usize;
      let offset = self.block_offsets[b_idx];
      let bitpacker = BitPacker4x::new();
      let _ = bitpacker.decompress(
        &self.patch.packet_data[offset..],
        &mut self.current_block,
        num_bits as u8,
      );

      // Determine start index in this block
      // If it is the very last block (overall), we start at (total-1)%128
      // If it is any other block, we start at 127.
      let total_blocks = self.patch.num_bits_per_block.len();
      if b_idx == total_blocks - 1 {
        self.in_block_idx = ((self.patch.original_len - 1) % BitPacker4x::BLOCK_LEN) as isize;
      } else {
        self.in_block_idx = (BitPacker4x::BLOCK_LEN - 1) as isize;
      }
    }

    let val = self.current_block[self.in_block_idx as usize];
    self.current_count += 1;
    self.in_block_idx -= 1;

    if self.in_block_idx < 0 {
      self.block_idx -= 1;
      // Next call will reload
    }

    Some(val)
  }
}
