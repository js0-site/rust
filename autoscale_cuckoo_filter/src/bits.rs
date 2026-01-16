//! Bit array for compact fingerprint storage.
//! 用于紧凑指纹存储的位数组

/// Bit array with fast u64 read/write operations.
/// 支持快速 u64 读写操作的位数组
#[derive(Debug, Clone, bitcode::Encode, bitcode::Decode)]
pub struct Bits(Vec<u8>);

/// Padding bytes for safe u64 unaligned reads.
/// 用于安全 u64 非对齐读取的填充字节数
const PADDING: usize = std::mem::size_of::<u64>();

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
    let byte_idx = pos >> 3;
    let bit_off = pos & 7;
    // SAFETY: padding ensures we won't read past buffer
    // 安全性：填充确保不会读取超出缓冲区
    let raw = unsafe { self.0.as_ptr().add(byte_idx).cast::<u64>().read_unaligned() };
    (raw >> bit_off) & mask
  }

  /// Read raw u64 at bit position (no masking).
  /// 在位位置读取原始 u64（不掩码）
  #[inline(always)]
  pub fn read_raw(&self, pos: usize) -> u64 {
    let byte_idx = pos >> 3;
    let bit_off = pos & 7;
    // SAFETY: padding ensures we won't read past buffer
    // 安全性：填充确保不会读取超出缓冲区
    let raw = unsafe { self.0.as_ptr().add(byte_idx).cast::<u64>().read_unaligned() };
    raw >> bit_off
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
    let byte_idx = pos >> 3;
    let bit_off = pos & 7;
    // SAFETY: padding ensures we won't write past buffer
    // 安全性：填充确保不会写入超出缓冲区
    let ptr = unsafe { self.0.as_mut_ptr().add(byte_idx).cast::<u64>() };
    let old = unsafe { ptr.read_unaligned() };
    let cleared = old & !(mask << bit_off);
    let new = cleared | ((val & mask) << bit_off);
    unsafe { ptr.write_unaligned(new) };
  }
}

#[cfg(test)]
mod test {
  use super::*;

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
  }
}
