use std::ptr;

/// Mask table for extracting low N bytes from u64
/// 掩码表：截取 u64 低 N 字节
const MASKS: [u64; 8] = [
  0x00000000000000FF,
  0x000000000000FFFF,
  0x0000000000FFFFFF,
  0x00000000FFFFFFFF,
  0x000000FFFFFFFFFF,
  0x0000FFFFFFFFFFFF,
  0x00FFFFFFFFFFFFFF,
  0xFFFFFFFFFFFFFFFF,
];

/// Calculate bytes needed for a u64 (1-8)
/// 计算 u64 所需字节数 (1-8)
#[inline(always)]
const fn byte_len(n: u64) -> u8 {
  // n|1 handles 0 case (lz=63) correctly
  // n|1 确保 0 映射到 1 字节
  let lz = (n | 1).leading_zeros();
  ((71 - lz) >> 3) as u8
}

/// Encode two u64 into buffer, return bytes written
/// 编码两个 u64 到缓冲区，返回写入字节数
///
/// Forces Little Endian for cross-platform compatibility
/// 强制小端序以保证跨平台兼容
#[inline(always)]
pub fn encode(n1: u64, n2: u64, buf: &mut [u8]) -> usize {
  let len1 = byte_len(n1);
  let len2 = byte_len(n2);
  let tag = ((len1 - 1) << 4) | (len2 - 1);

  unsafe {
    let ptr = buf.as_mut_ptr();
    *ptr = tag;

    // Write LE bytes, overlap write overwrites high zero-padding
    // 写入小端字节，覆盖写会覆盖高位零填充
    ptr::write_unaligned(ptr.add(1) as *mut u64, n1.to_le());
    ptr::write_unaligned(ptr.add(1 + len1 as usize) as *mut u64, n2.to_le());
  }

  (1 + len1 + len2) as usize
}

/// Decode two u64 from buffer, return (n1, n2, bytes_consumed)
/// 从缓冲区解码两个 u64，返回 (n1, n2, 消耗字节数)
///
/// SAFETY: buf must have 8 bytes padding after actual data
/// 安全性：buf 实际数据后需有 8 字节 padding
#[inline(always)]
pub fn decode(buf: &[u8]) -> (u64, u64, usize) {
  unsafe {
    let ptr = buf.as_ptr();
    let tag = *ptr;

    let len1_idx = ((tag >> 4) & 0x07) as usize;
    let len2_idx = (tag & 0x07) as usize;
    let len1 = len1_idx + 1;
    let len2 = len2_idx + 1;

    // Read LE bytes -> native u64 -> mask high garbage
    // 读取小端字节 -> 本机 u64 -> 掩码清除高位垃圾
    let raw_n1 = ptr::read_unaligned(ptr.add(1) as *const u64);
    let n1 = u64::from_le(raw_n1) & *MASKS.get_unchecked(len1_idx);

    let raw_n2 = ptr::read_unaligned(ptr.add(1 + len1) as *const u64);
    let n2 = u64::from_le(raw_n2) & *MASKS.get_unchecked(len2_idx);

    (n1, n2, 1 + len1 + len2)
  }
}
