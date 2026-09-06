//! Bit array for compact fingerprint storage.
//! 用于紧凑指纹存储的位数组

use std::mem;

/// Bit array with fast u64 read/write operations.
/// 支持快速 u64 读写操作的位数组
#[derive(Debug, Clone, bitcode::Encode, bitcode::Decode)]
pub struct Bits(Vec<u8>);

/// Padding bytes for safe u128 unaligned reads.
/// 用于安全 u128 非对齐读取的填充字节数
const PADDING: usize = mem::size_of::<u128>();

impl Bits {
  /// Create new bit array with given size hint (in bits).
  /// 创建指定大小（位）的新位数组
  pub fn new(size_hint: usize) -> Self {
    let len = size_hint.div_ceil(8) + PADDING;
    Bits(vec![0; len])
  }

  /// Returns the number of bits (excluding padding).
  /// 返回位数（不含填充）
  #[inline]
  pub fn len(&self) -> usize {
    (self.0.len().saturating_sub(PADDING)) * 8
  }

  /// Read unsigned integer at given bit position.
  /// 在指定位位置读取无符号整数
  #[inline(always)]
  #[cfg(test)]
  pub fn get_uint(&self, pos: usize, size: usize) -> u64 {
    // Prevent overflow when size >= 64
    // 当 size >= 64 时防止溢出
    let mask = if size >= 64 {
      u64::MAX
    } else {
      (1u64 << size) - 1
    };
    self.get_uint_masked(pos, mask)
  }

  /// Read unsigned integer with precomputed mask.
  /// 使用预计算的掩码读取无符号整数
  #[inline(always)]
  pub fn get_uint_masked(&self, pos: usize, mask: u64) -> u64 {
    self.read_raw(pos) & mask
  }

  /// Read raw u64 at bit position (no masking).
  /// 在位位置读取原始 u64（不掩码）
  ///
  /// Uses a 128-bit unaligned load so that any 64-bit window starting at `pos`
  /// is fully captured without high-bit truncation, even when `pos & 7 != 0`.
  /// 使用 128 位非对齐加载，确保即使位偏移非字节对齐，从 pos 起始的 64 位窗口亦完整保留不被截断。
  #[inline(always)]
  pub fn read_raw(&self, pos: usize) -> u64 {
    let byte_idx = pos >> 3;
    let bit_off = pos & 7;
    debug_assert!(
      byte_idx + mem::size_of::<u128>() <= self.0.len(),
      "read_raw: byte_idx out of bounds"
    );
    // SAFETY: padding ensures 16-byte read won't read past buffer
    // 安全性：填充确保 16 字节读取不会超出缓冲区
    let raw = unsafe {
      let ptr = self.0.as_ptr().add(byte_idx).cast::<[u8; 16]>();
      u128::from_le_bytes(*ptr)
    };
    (raw >> bit_off) as u64
  }

  /// Write unsigned integer at given bit position.
  /// 在指定位位置写入无符号整数
  #[inline(always)]
  #[cfg(test)]
  pub fn set_uint(&mut self, pos: usize, size: usize, val: u64) {
    // Prevent overflow when size >= 64
    // 当 size >= 64 时防止溢出
    let mask = if size >= 64 {
      u64::MAX
    } else {
      (1u64 << size) - 1
    };
    self.set_uint_masked(pos, mask, val);
  }

  /// Write unsigned integer with precomputed mask.
  /// 使用预计算的掩码写入无符号整数
  #[inline(always)]
  pub fn set_uint_masked(&mut self, pos: usize, mask: u64, val: u64) {
    let _ = self.swap_uint_masked(pos, mask, val);
  }

  /// Swap unsigned integer with precomputed mask in a single read-modify-write.
  /// 使用预计算掩码在单次读-改-写中交换无符号整数
  #[inline(always)]
  pub fn swap_uint_masked(&mut self, pos: usize, mask: u64, val: u64) -> u64 {
    let byte_idx = pos >> 3;
    let bit_off = pos & 7;
    debug_assert!(
      bit_off + 64 - mask.leading_zeros() as usize <= 64,
      "bit range must fit in u64"
    );
    debug_assert!(
      byte_idx + mem::size_of::<u64>() <= self.0.len(),
      "swap_uint_masked: byte_idx out of bounds"
    );
    // SAFETY: padding ensures we won't read or write past buffer
    // 安全性：填充确保读写不会超出缓冲区
    let ptr = unsafe { self.0.as_mut_ptr().add(byte_idx).cast::<[u8; 8]>() };
    let raw = u64::from_le_bytes(unsafe { *ptr });
    let old = (raw >> bit_off) & mask;
    let cleared = raw & !(mask << bit_off);
    let new = cleared | ((val & mask) << bit_off);
    unsafe { *ptr = new.to_le_bytes() };
    old
  }
}

#[cfg(test)]
mod test {
  use super::Bits;

  #[test]
  fn basic_ops() {
    let mut bits = Bits::new(12345);
    assert!(bits.len() >= 12344);

    assert_eq!(bits.get_uint(0, 1), 0);
    bits.set_uint(0, 1, 1);
    assert_eq!(bits.get_uint(0, 1), 1);

    assert_eq!(bits.get_uint(333, 10), 0);
    bits.set_uint(333, 10, 0b10_1101_0001);
    assert_eq!(bits.get_uint(333, 10), 0b10_1101_0001);

    assert_eq!(bits.get_uint(335, 4), 0b0100);
    bits.set_uint(335, 4, 0b1010);
    assert_eq!(bits.get_uint(335, 4), 0b1010);
    assert_eq!(bits.get_uint(333, 10), 0b10_1110_1001);
  }

  #[test]
  fn high_bits() {
    let mut bits = Bits::new(320);
    assert!(bits.len() >= 320);

    assert_eq!(bits.get_uint(290, 5), 0);
    bits.set_uint(290, 5, 31);
    assert_eq!(bits.get_uint(290, 5), 31);
    bits.set_uint(290, 5, 21);
    assert_eq!(bits.get_uint(290, 5), 21);
    let old = bits.swap_uint_masked(290, 31, 15);
    assert_eq!(old, 21);
    assert_eq!(bits.get_uint(290, 5), 15);
  }

  #[test]
  fn read_raw_unaligned_64bits() {
    let mut bits = Bits::new(512);
    // Write 56-bit value at bit 62 (byte 7, bit_off 6)
    // 在第 62 位（第 7 字节，偏移 6）写入 56 位数据
    let val56 = 0x00ff_eedd_ccbb_aa99u64;
    let mask56 = (1u64 << 56) - 1;
    bits.set_uint_masked(62, mask56, val56);

    let raw = bits.read_raw(62);
    assert_eq!(
      raw & mask56,
      val56,
      "read_raw at unaligned offset must not truncate upper bits"
    );
  }
}
