use bytes::{Buf, Bytes, BytesMut};
use num_traits::{AsPrimitive, Unsigned};

#[derive(Debug, PartialEq)]
pub enum DecodeError {
  /// 输入数据不足，无法完成解码
  InsufficientData,
  /// 输入数据超过允许的最大长度
  Overlength,
}

impl std::error::Error for DecodeError {}

impl std::fmt::Display for DecodeError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      DecodeError::InsufficientData => write!(f, "Insufficient data to decode"),
      DecodeError::Overlength => write!(f, "Input data exceeds maximum allowed length"),
    }
  }
}

/// 对无符号整型进行 protobuf varint 编码
pub fn encode<T, const MAX_LEN: usize>(value: T, flag: T) -> Vec<u8>
where
  T: Unsigned
    + Copy
    + PartialEq
    + From<u8>
    + Default
    + core::ops::Shr<usize, Output = T>
    + core::ops::BitAnd<T, Output = T>
    + AsPrimitive<u8>,
{
  let mut result = Vec::with_capacity(MAX_LEN); // 最多需要10个字节
  let mut v = value;
  let zero = Default::default();

  loop {
    let byte_value = v & flag;
    let mut byte: u8 = byte_value.as_(); // 取低7位
    v = v >> 7; // 右移7位

    if v != zero {
      byte |= 0x80; // 设置最高位表示还有后续字节
    }

    result.push(byte);

    if v == zero {
      break; // 如果没有更多字节，退出循环
    }
  }

  result
}

/// 对无符号整型进行 protobuf varint 解码
pub fn decode<U, const MAX_LEN: usize>(input: &[u8]) -> Result<(U, usize), DecodeError>
where
  U: Unsigned
    + Copy
    + PartialEq
    + From<u8>
    + Default
    + core::ops::Shl<usize, Output = U>
    + core::ops::BitAnd<U, Output = U>
    + std::ops::BitOrAssign,
{
  let mut result: U = Default::default();
  let mut shift = 0;
  let mut pos = 0;

  while pos < input.len() {
    let byte = input[pos];
    pos += 1;

    let t: U = (byte & 0x7F).into();
    // 将低7位添加到结果中
    result |= t << shift;

    // 检查是否有后续字节
    if (byte & 0x80) == 0 {
      // 解码完成
      return Ok((result, pos));
    }

    shift += 7;

    // 检查是否超出最大长度
    if shift >= MAX_LEN * 7 {
      return Err(DecodeError::Overlength);
    }
  }

  // 输入数据不足，无法完成解码
  Err(DecodeError::InsufficientData)
}

/// 对 u32 进行 protobuf varint 编码
pub fn encode_u32(value: u32) -> Vec<u8> {
  encode::<_, 5>(value, 0x7F) // u32 最多需要5个字节
}

/// 从字节切片解码 protobuf varint 为 u32
/// 返回 (解码的 u32 值, 使用的字节数)
pub fn decode_u32(input: &[u8]) -> Result<(u32, usize), DecodeError> {
  decode::<u32, 5>(input)
}

pub fn get_u32_bin(buf: &mut BytesMut) -> Result<Option<Bytes>, DecodeError> {
  Ok(match decode_u32(buf) {
    Ok((_n, len)) => Some(buf.split_to(len).freeze()),
    Err(DecodeError::InsufficientData) => {
      // 数据不足，等待更多数据
      None
    }
    Err(err) => {
      return Err(err);
    }
  })
}

pub fn get_n<T>(
  buf: &mut BytesMut,
  decode_n: impl FnOnce(&[u8]) -> Result<(T, usize), DecodeError>,
) -> Result<Option<T>, DecodeError> {
  Ok(match decode_n(buf) {
    Ok((n, len)) => {
      buf.advance(len);
      Some(n)
    }
    Err(DecodeError::InsufficientData) => {
      // 数据不足，等待更多数据
      None
    }
    Err(err) => {
      return Err(err);
    }
  })
}

pub fn get_u32(buf: &mut BytesMut) -> Result<Option<u32>, DecodeError> {
  get_n(buf, decode_u32)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_encode_decode_u32() {
    let test_values = vec![0, 1, 127, 128, 255, 256, 1000, 10000, u32::MAX];

    for value in test_values {
      let encoded = encode_u32(value);
      let (decoded, len) = decode_u32(&encoded).unwrap();
      assert_eq!(decoded, value);
      assert_eq!(len, encoded.len());
    }
  }

  #[test]
  fn test_decode_error() {
    // 测试不完整的数据
    let incomplete_data = vec![0xFF, 0xFF, 0xFF]; // 未以最高位为0的字节结尾
    assert_eq!(
      decode_u32(&incomplete_data),
      Err(DecodeError::InsufficientData)
    );

    // 测试超过u32最大长度的数据
    let too_long_data = vec![
      0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01,
    ]; // 11字节
    assert_eq!(decode_u32(&too_long_data), Err(DecodeError::Overlength));
  }
}
