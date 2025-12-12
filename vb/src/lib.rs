#![cfg_attr(docsrs, feature(doc_cfg))]

use std::borrow::Borrow;

#[derive(thiserror::Error, Debug)]
pub enum Error {
  #[error("Incomplete vbyte sequence")]
  VbyteNoEnd,
  #[error("Vbyte overflow")]
  VbyteOverflow,
}

pub type Result<T> = std::result::Result<T, Error>;

/// Decode a single variable-byte encoded integer from the input.
/// Returns the value and the number of bytes consumed.
///
/// 从输入中解码一个变长编码的整数。
/// 返回该值和消耗的字节数。
pub fn d(input: impl AsRef<[u8]>) -> Result<(u64, usize)> {
  let mut value: u64 = 0;
  let mut shift = 0;
  let mut consumed = 0;

  for byte in input.as_ref() {
    let data = (byte & 0x7F) as u64;
    value |= data << shift;
    consumed += 1;

    if byte & 0x80 == 0 {
      // MSB 为 0，结束
      return Ok((value, consumed));
    }

    shift += 7;
    if shift >= 64 {
      return Err(Error::VbyteOverflow);
    }
  }

  Err(Error::VbyteNoEnd)
}

/// Encodes a single `u64` into variable-byte format and appends to the buffer.
/// This function does not clear the buffer; it appends the encoded bytes to the end.
///
/// 将单个 `u64` 编码为变长格式并追加到缓冲区。
/// 此函数不会清空缓冲区；它会将编码后的字节追加到末尾。
#[inline]
pub fn e(mut value: u64, bytes: &mut Vec<u8>) {
  loop {
    let mut byte = (value & 0x7F) as u8; // 取低7位
    value >>= 7;
    if value != 0 {
      byte |= 0x80; // 高位1表示还有后续字节
    }
    bytes.push(byte);
    if value == 0 {
      break;
    }
  }
}

/// Encodes a list of `u64` integers into variable-byte format.
///
/// 将 `u64` 整数列表编码为变长格式。
pub fn e_li<U: Borrow<u64>>(li: impl IntoIterator<Item = U>) -> Vec<u8> {
  let mut result = Vec::new();
  for num in li {
    e(*num.borrow(), &mut result);
  }
  result
}

/// Decodes a list of variable-byte encoded integers from the input.
///
/// 从输入中解码变长编码的整数列表。
pub fn d_li(data: impl AsRef<[u8]>) -> Result<Vec<u64>> {
  let bytes = data.as_ref();
  let len = bytes.len();
  let mut result = Vec::with_capacity(len / 2); // Heuristic estimate
  let mut offset = 0;

  while offset < len {
    let (num, consumed) = d(&bytes[offset..])?;
    result.push(num);
    offset += consumed;
  }

  Ok(result)
}

/// Encodes a strictly increasing sequence of `u64` integers using differential encoding (delta encoding) combined with variable-byte encoding.
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

  // First element
  e(li[0], &mut result);

  // Subsequent elements (deltas)
  for i in 1..li.len() {
    e(li[i] - li[i - 1], &mut result);
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
pub fn d_diff(vs: impl AsRef<[u8]>) -> Result<Vec<u64>> {
  let mut li = d_li(vs)?;
  if li.len() >= 2 {
    for i in 1..li.len() {
      li[i] += li[i - 1];
    }
  }

  Ok(li)
}
