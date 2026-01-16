use core::hash::Hasher;

/// RapidHash finalization mix implementation (hand-written const version)
/// RapidHash 终结混淆实现 (手写 const 版本)
///
/// This is a finalization step for a fast non-cryptographic hash algorithm, providing excellent avalanche properties.
/// The hand-written version ensures extreme performance and const compatibility on the filter hot path.
///
/// 这是一个快速非加密哈希算法的终结步骤，提供优秀的雪崩特性。
/// 手写版本确保了在过滤器热路径上的极致性能和 const 兼容性。
#[inline(always)]
pub const fn mix64(k: u64) -> u64 {
  // Minimal 2-step mixer: MUL + XOR-Shift
  // 最小化2步混合器：乘法 + 异或移位
  let h = k.wrapping_mul(0xff51afd7ed558ccd);
  h ^ (h >> 33)
}

/// Applies a finalization mix to a randomly-seeded key.
/// 对随机种子密钥应用终结混淆
#[inline(always)]
pub const fn mix(key: u64, seed: u64) -> u64 {
  mix64(key.wrapping_add(seed))
}

/// Updates the pseudo-random seed using a Weyl sequence.
/// 使用 Weyl 序列更新伪随机种子
#[inline(always)]
pub const fn rand(seed: &mut u64) -> u64 {
  *seed = (*seed).wrapping_add(0x9e37_79b9_7f4a_7c15);
  *seed
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
    // Initialize official Hasher with common seed
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
