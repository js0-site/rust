//! Variable byte encoding library for u64 integers.
//! 变长字节编码库，用于 u64 整数。

#![cfg_attr(docsrs, feature(doc_cfg))]

use thiserror::Error;

// ============================================================================
// Error Definitions
// 错误定义
// ============================================================================

#[derive(Error, Debug)]
pub enum Error {
  /// Incomplete vbyte sequence - input ended before a complete value was decoded.
  /// 不完整的变长字节序列 - 输入在解码完整值之前结束。
  #[error("Incomplete vbyte sequence")]
  VbyteNoEnd,

  /// Vbyte overflow - the decoded value exceeds u64::MAX.
  /// 变长字节溢出 - 解码值超过 u64::MAX。
  #[error("Vbyte overflow")]
  VbyteOverflow,
}

pub type Result<T> = std::result::Result<T, Error>;

// ============================================================================
// Constants
// 常量
// ============================================================================

/// Maximum bytes needed to encode a u64 in variable-byte format.
/// u64 变长编码所需的最大字节数。
const MAX_VARINT_BYTES: usize = 10;

/// Continuation bit mask (high bit set indicates more bytes follow).
/// 延续位掩码（高位置 1 表示后续还有字节）。
const CONTINUATION_BIT: u8 = 0x80;

/// Data mask (lower 7 bits contain actual data).
/// 数据掩码（低 7 位包含实际数据）。
const DATA_MASK: u8 = 0x7F;

/// Bits per byte in variable-byte encoding.
/// 变长编码中每字节的有效位数。
const BITS_PER_BYTE: u32 = 7;

// ============================================================================
// Decoding Functions
// 解码函数
// ============================================================================

/// Decode a single variable-byte encoded integer from the input.
/// Returns the value and the number of bytes consumed.
///
/// 从输入中解码一个变长编码的整数。
/// 返回该值和消耗的字节数。
#[inline(always)]
pub fn d(bytes: &[u8]) -> Result<(u64, usize)> {
  let mut offset = 0;
  let value = d_offset(bytes, &mut offset)?;
  Ok((value, offset))
}

/// Decode a single variable-byte encoded integer from the buffer at the given offset.
/// Updates the offset to point after the decoded value.
/// Highly optimized with unsafe pointer arithmetic and bounds check elimination.
///
/// 从给定偏移量的缓冲区解码单个变长编码整数。
/// 更新偏移量以指向解码值之后。
/// 高度优化的解码函数，使用了 unsafe 指针运算并消除了热点路径的边界检查。
#[inline(always)]
pub fn d_offset(buf: &[u8], offset: &mut usize) -> Result<u64> {
  let pos = *offset;
  let len = buf.len();

  // Boundary check for start
  // 起始边界检查
  if pos >= len {
    return Err(Error::VbyteNoEnd);
  }

  // SAFETY: pos < len checked above
  // 安全性：上面已检查 pos < len
  let first_byte = unsafe { *buf.get_unchecked(pos) };

  // Fast path: single byte value < 128 (most common case, ~60% of real data)
  // 快速路径：单字节值 < 128（最常见情况，约占真实数据的 60%）
  if first_byte < CONTINUATION_BIT {
    *offset = pos + 1;
    return Ok(u64::from(first_byte));
  }

  // Multi-byte decoding
  // 多字节解码
  let remaining = len - pos;

  // Fast path: >= 10 bytes remaining, no bounds check needed in loop
  // 快速路径：剩余 >= 10 字节，循环内无需边界检查
  if remaining >= MAX_VARINT_BYTES {
    // SAFETY: We verified remaining >= 10, so all accesses within 10 bytes are safe
    // 安全性：已验证 remaining >= 10，因此 10 字节内的所有访问都是安全的
    unsafe { decode_multi_byte_unchecked(buf, offset, pos) }
  } else {
    // Slow path: < 10 bytes remaining, must check bounds
    // 慢速路径：剩余 < 10 字节，必须检查边界
    decode_multi_byte_checked(buf, offset, pos, len)
  }
}

/// Decode multi-byte varint without bounds checking.
///
/// # Safety
/// Caller must ensure buf has at least 10 bytes from pos.
///
/// 无边界检查的多字节变长整数解码。
///
/// # 安全性
/// 调用者必须确保 buf 从 pos 开始至少有 10 字节。
#[inline(always)]
unsafe fn decode_multi_byte_unchecked(buf: &[u8], offset: &mut usize, pos: usize) -> Result<u64> {
  // SAFETY: Caller guarantees buf has at least 10 bytes from pos
  // 安全性：调用者保证 buf 从 pos 开始至少有 10 字节
  let ptr = unsafe { buf.as_ptr().add(pos) };

  // Manually unrolled decode loop for better performance
  // 手动展开解码循环以获得更好的性能
  macro_rules! decode_byte {
    ($i:expr, $shift:expr, $value:expr) => {{
      // SAFETY: $i < 10 and caller guarantees 10 bytes available
      // 安全性：$i < 10 且调用者保证有 10 字节可用
      let byte = unsafe { *ptr.add($i) };
      if byte < CONTINUATION_BIT {
        *offset = pos + $i + 1;
        return Ok($value | (u64::from(byte) << $shift));
      }
      $value | (u64::from(byte & DATA_MASK) << $shift)
    }};
  }

  let v = decode_byte!(0, 0, 0u64);
  let v = decode_byte!(1, 7, v);
  let v = decode_byte!(2, 14, v);
  let v = decode_byte!(3, 21, v);
  let v = decode_byte!(4, 28, v);
  let v = decode_byte!(5, 35, v);
  let v = decode_byte!(6, 42, v);
  let v = decode_byte!(7, 49, v);
  let v = decode_byte!(8, 56, v);

  // 10th byte: only bit 63 allowed, no continuation
  // 第 10 字节：仅允许第 63 位，无延续
  // SAFETY: caller guarantees 10 bytes available
  // 安全性：调用者保证有 10 字节可用
  let byte = unsafe { *ptr.add(9) };
  if byte > 1 {
    return Err(Error::VbyteOverflow);
  }
  *offset = pos + MAX_VARINT_BYTES;
  Ok(v | (u64::from(byte) << 63))
}

/// Decode multi-byte varint with bounds checking.
///
/// 带边界检查的多字节变长整数解码。
fn decode_multi_byte_checked(
  buf: &[u8],
  offset: &mut usize,
  mut pos: usize,
  len: usize,
) -> Result<u64> {
  let mut value = 0u64;
  let mut shift = 0u32;

  while pos < len {
    // SAFETY: pos < len checked by loop condition
    // 安全性：循环条件已检查 pos < len
    let byte = unsafe { *buf.get_unchecked(pos) };
    pos += 1;

    if byte < CONTINUATION_BIT {
      value |= u64::from(byte) << shift;
      *offset = pos;
      return Ok(value);
    }

    value |= u64::from(byte & DATA_MASK) << shift;
    shift += BITS_PER_BYTE;

    // After 9 bytes (63 bits), need 10th byte
    // 9 字节（63 位）后，需要第 10 字节
    if shift >= 63 {
      if pos >= len {
        return Err(Error::VbyteNoEnd);
      }
      // SAFETY: pos < len checked above
      // 安全性：上面已检查 pos < len
      let byte = unsafe { *buf.get_unchecked(pos) };
      // 10th byte: only bit 63 allowed, no continuation
      // 第 10 字节：仅允许第 63 位，无延续
      if byte > 1 {
        return Err(Error::VbyteOverflow);
      }
      value |= u64::from(byte) << 63;
      *offset = pos + 1;
      return Ok(value);
    }
  }

  Err(Error::VbyteNoEnd)
}

// ============================================================================
// Encoding Functions
// 编码函数
// ============================================================================

/// Encodes a single `u64` into variable-byte format and appends to the buffer.
/// This function does not clear the buffer; it appends the encoded bytes to the end.
/// Optimized to avoid repeated capacity checks and len updates.
///
/// 将单个 `u64` 编码为变长格式并追加到缓冲区。
/// 此函数不会清空缓冲区；它会将编码后的字节追加到末尾。
/// 优化后避免重复的容量检查和长度更新。
#[inline(always)]
pub fn e(value: u64, buf: &mut Vec<u8>) {
  // Reserve space for worst case (10 bytes for u64 max).
  // This is cheap if capacity is already sufficient.
  // 预留空间（最坏情况 10 字节），如果容量足够，这几乎无开销。
  buf.reserve(MAX_VARINT_BYTES);

  // SAFETY: We reserved enough space (10 bytes).
  // 安全性：我们已预留足够空间（10 字节）。
  unsafe {
    let len = buf.len();
    let ptr = buf.as_mut_ptr().add(len);

    // Use index-based loop for better optimization
    // 使用基于索引的循环以获得更好的优化
    let bytes_written = if value < u64::from(CONTINUATION_BIT) {
      // Fast path: single byte (most common)
      // 快速路径：单字节（最常见）
      *ptr = value as u8;
      1
    } else {
      encode_multi_byte(ptr, value)
    };

    buf.set_len(len + bytes_written);
  }
}

/// Encode multi-byte varint, returns number of bytes written.
/// Uses leading_zeros for fast byte count calculation and unrolled writes.
///
/// # Safety
/// ptr must have at least 10 bytes of valid memory.
///
/// 编码多字节变长整数，返回写入的字节数。
/// 使用 leading_zeros 快速计算字节数，并展开写入操作。
///
/// # 安全性
/// ptr 必须有至少 10 字节的有效内存。
#[inline(always)]
unsafe fn encode_multi_byte(ptr: *mut u8, value: u64) -> usize {
  // Calculate number of bytes needed using CLZ instruction
  // 使用 CLZ 指令计算所需字节数
  let bits = 64 - value.leading_zeros() as usize;
  let num_bytes = bits.div_ceil(BITS_PER_BYTE as usize);

  // Unrolled write based on byte count
  // 根据字节数展开写入
  // SAFETY: caller guarantees 10 bytes available, num_bytes <= 10
  // 安全性：调用者保证有 10 字节可用，num_bytes <= 10
  match num_bytes {
    2 => unsafe {
      *ptr = (value as u8) | CONTINUATION_BIT;
      *ptr.add(1) = (value >> 7) as u8;
    },
    3 => unsafe {
      *ptr = (value as u8) | CONTINUATION_BIT;
      *ptr.add(1) = ((value >> 7) as u8) | CONTINUATION_BIT;
      *ptr.add(2) = (value >> 14) as u8;
    },
    4 => unsafe {
      *ptr = (value as u8) | CONTINUATION_BIT;
      *ptr.add(1) = ((value >> 7) as u8) | CONTINUATION_BIT;
      *ptr.add(2) = ((value >> 14) as u8) | CONTINUATION_BIT;
      *ptr.add(3) = (value >> 21) as u8;
    },
    5 => unsafe {
      *ptr = (value as u8) | CONTINUATION_BIT;
      *ptr.add(1) = ((value >> 7) as u8) | CONTINUATION_BIT;
      *ptr.add(2) = ((value >> 14) as u8) | CONTINUATION_BIT;
      *ptr.add(3) = ((value >> 21) as u8) | CONTINUATION_BIT;
      *ptr.add(4) = (value >> 28) as u8;
    },
    _ => {
      // 6-10 bytes: use loop for rare large values
      // 6-10 字节：对罕见的大值使用循环
      let mut v = value;
      for i in 0..num_bytes - 1 {
        unsafe { *ptr.add(i) = (v as u8) | CONTINUATION_BIT };
        v >>= 7;
      }
      unsafe { *ptr.add(num_bytes - 1) = v as u8 };
    }
  }

  num_bytes
}

/// Encodes a list of `u64` integers into variable-byte format.
///
/// 将 `u64` 整数列表编码为变长格式。
#[inline]
pub fn e_li(li: impl IntoIterator<Item = u64>) -> Vec<u8> {
  let iter = li.into_iter();
  let (lower, _) = iter.size_hint();
  // Heuristic: assume average 2 bytes per integer to avoid frequent resizing
  // 启发式：假设每个整数平均占用 2 字节，以避免频繁调整大小
  let mut result = Vec::with_capacity(lower.saturating_mul(2));
  for num in iter {
    e(num, &mut result);
  }
  result
}

/// Decodes a list of variable-byte encoded integers from the input.
///
/// 从输入中解码变长编码的整数列表。
#[inline]
pub fn d_li(bytes: &[u8]) -> Result<Vec<u64>> {
  let len = bytes.len();
  // Heuristic: assume average 2 bytes per int
  // 启发式：假设平均每个整数 2 字节
  let mut result = Vec::with_capacity(len / 2);
  let mut offset = 0;

  while offset < len {
    // Pass reference to offset to avoid slicing overhead
    // 传递偏移量引用以避免切片开销
    result.push(d_offset(bytes, &mut offset)?);
  }

  Ok(result)
}

// ============================================================================
// Differential Encoding (Feature: diff)
// 差分编码（功能：diff）
// ============================================================================

/// Encodes a strictly increasing sequence of `u64` integers using differential encoding
/// (delta encoding) combined with variable-byte encoding.
/// This reduces the serialized size by storing the differences between consecutive values.
///
/// 使用差分编码（增量编码）结合变长编码对严格递增的 `u64` 整数序列进行编码。
/// 通过存储连续值之间的差值来减少序列化后的大小，具有压缩效果。
#[cfg(feature = "diff")]
#[cfg_attr(docsrs, doc(cfg(feature = "diff")))]
pub fn e_diff(li: impl AsRef<[u64]>) -> Vec<u8> {
  let li = li.as_ref();
  if li.is_empty() {
    return Vec::new();
  }

  let mut result = Vec::with_capacity(li.len());
  let mut prev = 0;

  for &curr in li {
    // Sorting check is O(N). In release mode, we assume input is correct for performance.
    // In debug, we panic to alert developer.
    // 排序检查是 O(N)。在 Release 模式下，为了性能我们假设输入正确。
    // 在 Debug 模式下，我们 panic 以提醒开发者。
    debug_assert!(curr >= prev, "e_diff requires strictly increasing sequence");

    // Wrapping sub is safe: if curr < prev in release, it wraps to huge u64,
    // which preserves "no crash" constraint but data is garbage.
    // 环绕减法是安全的：如果 Release 模式下 curr < prev，它会环绕为巨大的 u64，
    // 这符合"不崩溃"的约束，但数据是无意义的。
    e(curr.wrapping_sub(prev), &mut result);
    prev = curr;
  }

  result
}

/// Decodes a sequence of integers encoded with `e_diff`.
/// Reconstructs the original increasing sequence from the differences.
///
/// 解码使用 `e_diff` 编码的整数序列。
/// 从差值中重建原始的递增序列。
#[cfg(feature = "diff")]
#[cfg_attr(docsrs, doc(cfg(feature = "diff")))]
pub fn d_diff(bytes: &[u8]) -> Result<Vec<u64>> {
  let len = bytes.len();
  // Conservative estimate
  // 保守估计
  let mut result = Vec::with_capacity(len);
  let mut offset = 0;
  let mut prev = 0u64;

  while offset < len {
    let delta = d_offset(bytes, &mut offset)?;

    // Check overflow for correctness (data integrity).
    // 检查溢出以确保正确性（数据完整性）。
    let val = prev.checked_add(delta).ok_or(Error::VbyteOverflow)?;
    result.push(val);
    prev = val;
  }

  Ok(result)
}
