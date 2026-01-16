use core::hash::Hasher;

const MIX_C1: u64 = 0xff51_afd7_ed55_8ccd;

/// RapidHash / WyHash style 64-bit mixer (128-bit product based).
/// 基于 RapidHash / WyHash 的 128 位乘法混淆实现。
///
/// RapidHash 和 WyHash 均使用这种 (A * B) -> lo ^ hi 的策略，
/// 利用 64x64->128 乘法指令实现极高的混合质量和速度。
/// 这是一个经受住时间考验的方案，完全满足 Binary Fuse Filter 的需求。
#[inline(always)]
pub const fn mix64(k: u64) -> u64 {
  // RapidHash/WyHash 核心混淆逻辑： (x * C) ^ ((x * C) >> 64)
  // 这里的常数选用之前使用的高质量常数 MIX_C1
  let r = (k as u128).wrapping_mul(MIX_C1 as u128);
  (r ^ (r >> 64)) as u64
}

/// A high-speed Hasher wrapper using the official hash crate.
/// 基于官方 hash crate 的高速度哈希器封装
///
/// Used for HashProxy to process non-u64 types.
/// 用于 HashProxy 处理非 u64 类型。
#[derive(Clone)]
pub struct RapidHasher(rapidhash::fast::RapidHasher<'static>);

impl Default for RapidHasher {
  #[inline(always)]
  fn default() -> Self {
    Self(rapidhash::fast::RapidHasher::new(MIX_C1))
  }
}

impl Hasher for RapidHasher {
  #[inline(always)]
  fn finish(&self) -> u64 {
    self.0.finish()
  }

  #[inline(always)]
  fn write(&mut self, bytes: &[u8]) {
    self.0.write(bytes);
  }

  #[inline(always)]
  fn write_u64(&mut self, i: u64) {
    self.0.write_u64(i);
  }
}

/// Fingerprint type constraint.
/// 指纹类型约束
pub trait Fingerprint:
  Default + Copy + PartialEq + core::ops::BitXorAssign + core::ops::BitXor<Output = Self> + 'static
{
  /// Zero value constant.
  /// 零值常量
  const ZERO: Self;

  /// Alignment requirement.
  /// 对齐要求
  const ALIGN: usize;

  /// Computes a fingerprint from a hash.
  /// 从哈希值计算指纹
  fn from_hash(hash: u64) -> Self;

  /// Converts a fingerprint slice to raw bytes.
  /// 将指纹切片转换为原始字节
  fn as_bytes(slice: &[Self]) -> &[u8];

  /// Safely converts raw bytes to a fingerprint slice.
  /// 从原始字节安全转换为指纹切片
  fn from_bytes(bytes: &[u8]) -> &[Self];
}

macro_rules! impl_fingerprint {
  ($ty:ty, $align:expr) => {
    impl Fingerprint for $ty {
      const ZERO: Self = 0;
      const ALIGN: usize = $align;

      #[inline(always)]
      fn from_hash(hash: u64) -> Self {
        // mix64 已提供充分雪崩效应，直接使用低位即可
        // mix64 already provides sufficient avalanche effect, just use low bits
        hash as Self
      }

      #[inline(always)]
      fn as_bytes(slice: &[Self]) -> &[u8] {
        let len = slice.len() * core::mem::size_of::<Self>();
        unsafe { core::slice::from_raw_parts(slice.as_ptr() as *const u8, len) }
      }

      #[inline(always)]
      #[allow(clippy::modulo_one)]
      fn from_bytes(bytes: &[u8]) -> &[Self] {
        let size = core::mem::size_of::<Self>();
        if Self::ALIGN > 1 {
          assert!(
            bytes.as_ptr() as usize % Self::ALIGN == 0,
            "Invalid alignment for fingerprint type"
          );
        }
        assert!(
          bytes.len() % size == 0,
          "Invalid byte length for fingerprint type"
        );
        unsafe { core::slice::from_raw_parts(bytes.as_ptr() as *const Self, bytes.len() / size) }
      }
    }
  };
}

impl_fingerprint!(u8, 1);
impl_fingerprint!(u16, 2);
impl_fingerprint!(u32, 4);
