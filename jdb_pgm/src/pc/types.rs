/// Fixed-point: Key = (index * slope_fp + intercept_fp) >> 32
/// 定点数：Key = (index * slope_fp + intercept_fp) >> 32
#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct CompactSegment {
  pub start_idx: u32,
  pub slope_fp: u64,
  pub intercept_fp: i64,
}

#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct BlockMeta {
  /// Bit offset in the residuals array
  /// 残差数组中的位偏移
  pub bit_offset: u32,
  /// Bit width for each element in the block
  /// 块中每个元素的位宽
  pub bit_width: u8,
}

pub const BLOCK_LEN: usize = 128;
