use core::hash::Hasher;

/// RapidHash 终结混淆实现 (手写 const 版本)
/// RapidHash finalization mix implementation (hand-written const version)
///
/// 这是一个快速非加密哈希算法的终结步骤，提供优秀的雪崩特性。
/// 手写版本确保了在过滤器热路径上的极致性能和 const 兼容性。
#[inline(always)]
pub const fn mix64(k: u64) -> u64 {
  // RapidHash 终结混淆 - 提供类似 MurmurHash3 的雪崩效应
  let mut h = k;
  h ^= h >> 33;
  h = h.wrapping_mul(0xff51afd7ed558ccd);
  h ^= h >> 33;
  h = h.wrapping_mul(0xc4ceb9fe1a85ec53);
  h ^= h >> 33;
  h
}

/// 对随机种子密钥应用终结混淆
/// Applies a finalization mix to a randomly-seeded key.
#[inline(always)]
pub const fn mix(key: u64, seed: u64) -> u64 {
  mix64(key.wrapping_add(seed))
}

/// 使用 Weyl 序列更新伪随机种子
/// Updates the pseudo-random seed using a Weyl sequence.
#[inline(always)]
pub const fn rand(seed: &mut u64) -> u64 {
  *seed = (*seed).wrapping_add(0x9e37_79b9_7f4a_7c15);
  *seed
}

/// 基于官方 hash crate 的高速度哈希器封装
/// A high-speed Hasher wrapper using the official hash crate.
///
/// 用于 HashProxy 处理非 u64 类型。
#[derive(Clone)]
pub struct RapidHasher(rapidhash::fast::RapidHasher<'static>);

impl Default for RapidHasher {
  #[inline(always)]
  fn default() -> Self {
    // 使用常用种子初始化官方 Hasher
    Self(rapidhash::fast::RapidHasher::new(0x2d358dccaa6c78a5))
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

/// 指纹类型约束
/// Fingerprint type constraint.
pub trait Fingerprint:
  Default + Copy + PartialEq + core::ops::BitXorAssign + core::ops::BitXor<Output = Self> + 'static
{
  /// 对齐要求
  /// Alignment requirement.
  const ALIGN: usize;

  /// 从哈希值计算指纹
  /// Computes a fingerprint from a hash.
  fn from_hash(hash: u64) -> Self;

  /// 将指纹切片转换为原始字节
  /// Converts a fingerprint slice to raw bytes.
  fn as_bytes(slice: &[Self]) -> &[u8];

  /// 从原始字节安全转换为指纹切片
  /// Safely converts raw bytes to a fingerprint slice.
  fn from_bytes(bytes: &[u8]) -> &[Self];
}

macro_rules! impl_fingerprint {
  ($ty:ty, $align:expr) => {
    impl Fingerprint for $ty {
      const ALIGN: usize = $align;

      #[inline(always)]
      fn from_hash(hash: u64) -> Self {
        (hash ^ (hash >> 32)) as Self
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
