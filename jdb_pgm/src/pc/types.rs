/// Fixed-point: Key = (index * slope_fp + intercept_fp) >> 32
/// 定点数：Key = (index * slope_fp + intercept_fp) >> 32
#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct CompactSegment {
  pub start_idx: u32,
  pub slope_fp: u64,
  pub intercept_fp: i64,
}

/// BlockMeta.seg_idx flag: block has exceptions
/// BlockMeta.seg_idx 标志位：块有异常值
pub const FLAG_HAS_EX: u32 = 0x8000_0000;

/// BlockMeta.seg_idx flag: block contains single segment (no segment crossing)
/// BlockMeta.seg_idx 标志位：块只包含单一段（无跨段）
pub const FLAG_SINGLE_SEG: u32 = 0x4000_0000;

/// BlockMeta.seg_idx mask: extract segment index (lower 30 bits)
/// BlockMeta.seg_idx 掩码：提取段索引（低 30 位）
pub const MASK_SEG_IDX: u32 = 0x3FFF_FFFF;

#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct BlockMeta {
  /// Bit offset in the residuals array
  /// 残差数组中的位偏移
  pub bit_offset: u32,
  /// Bit width for each element in the block
  /// 块中每个元素的位宽
  pub bit_width: u8,
  /// Index of the first segment covering this block (with flags in high bits)
  /// 覆盖该块起始位置的段索引（高位含标志位）
  pub seg_idx: u32,
  /// Start index in the exceptions array for this block
  /// 该块在异常值数组中的起始索引
  pub exception_offset: u32,
}

include!(concat!(env!("OUT_DIR"), "/pc_consts.rs"));
