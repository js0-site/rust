//! Type definitions for Pgm-Index
//! Pgm 索引类型定义

#![allow(clippy::cast_precision_loss, clippy::cast_lossless)]

use std::fmt::Debug;

/// Key trait for supported types
/// 支持的键类型约束
pub trait Key: Copy + Send + Sync + Ord + Debug + 'static {
  /// Convert to f64
  /// 转换为 f64
  fn as_f64(self) -> f64;
}

/// Trait for types that can be converted to Key and provide bytes reference
/// 可转换为 Key 并提供字节引用的类型 trait
pub trait ToKey<K: Key> {
  /// Convert to Key type
  /// 转换为 Key 类型
  fn to_key(&self) -> K;

  /// Get bytes reference
  /// 获取字节引用
  fn as_bytes(&self) -> &[u8];
}

macro_rules! impl_key {
  ($($t:ty),*) => {
    $(
      impl Key for $t {
        #[inline(always)]
        fn as_f64(self) -> f64 {
          self as f64
        }
      }
    )*
  };
}

impl_key!(
  u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize
);

/// Helper to convert bytes to u64 (big-endian, pad with 0)
/// 将字节转换为 u64（大端序，不足补0）
#[inline]
fn bytes_to_u64(bytes: &[u8]) -> u64 {
  let len = bytes.len().min(8);
  let mut buf = [0u8; 8];
  buf[..len].copy_from_slice(&bytes[..len]);
  u64::from_be_bytes(buf)
}

macro_rules! impl_to_key {
  // For slice types
  // 切片类型
  (slice: $($t:ty),*) => {
    $(
      impl ToKey<u64> for $t {
        #[inline]
        fn to_key(&self) -> u64 {
          bytes_to_u64(self.as_ref())
        }
        #[inline]
        fn as_bytes(&self) -> &[u8] {
          self.as_ref()
        }
      }
    )*
  };
}

impl_to_key!(slice: [u8], &[u8], Vec<u8>, Box<[u8]>);

impl<const N: usize> ToKey<u64> for [u8; N] {
  #[inline]
  fn to_key(&self) -> u64 {
    bytes_to_u64(self)
  }
  #[inline]
  fn as_bytes(&self) -> &[u8] {
    self
  }
}

impl<const N: usize> ToKey<u64> for &[u8; N] {
  #[inline]
  fn to_key(&self) -> u64 {
    bytes_to_u64(*self)
  }
  #[inline]
  fn as_bytes(&self) -> &[u8] {
    *self
  }
}

/// Linear segment: y = slope * x + intercept
/// 线性段：y = slope * x + intercept
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Segment<K: Key> {
  pub min_key: K,
  pub max_key: K,
  pub slope: f64,
  pub intercept: f64,
  pub start_idx: u32,
  pub end_idx: u32,
}

/// Index statistics
/// 索引统计信息
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
#[derive(Clone, Debug, Default)]
pub struct PgmStats {
  pub segments: usize,
  pub avg_segment_size: f64,
  pub memory_bytes: usize,
}
