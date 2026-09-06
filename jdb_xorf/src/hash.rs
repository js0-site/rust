use core::ops::{BitXor, BitXorAssign};

#[cfg(feature = "gxhash")]
pub use gxhash::GxHasher;
// The active backend is re-exported under the unified name `DefaultHasher`.
// 将启用的后端以统一名称 `DefaultHasher` 导出。
#[cfg(feature = "gxhash")]
pub use gxhash::GxHasher as DefaultHasher;

#[cfg(not(feature = "gxhash"))]
pub use self::mix_hasher::MixHasher;
#[cfg(not(feature = "gxhash"))]
pub use self::mix_hasher::MixHasher as DefaultHasher;
#[cfg(feature = "museair")]
pub use self::museair_hasher::MuseairHasher;

#[cfg(feature = "museair")]
mod museair_hasher {
  use core::hash::Hasher as CoreHasher;

  // Unified backend name: the wrapped hasher is simply `Hasher`.
  // 统一后端命名：被包装的哈希器直接叫 `Hasher`。
  use museair::bfast::Hasher;

  /// High-speed portable incremental Hasher using MuseAir (BFast variant).
  /// 基于 MuseAir (BFast 变体) 的高性能可移植增量哈希器。
  #[derive(Clone, Debug)]
  pub struct MuseairHasher(Hasher);

  impl Default for MuseairHasher {
    #[inline(always)]
    fn default() -> Self {
      Self(Hasher::with_seed(0))
    }
  }

  impl CoreHasher for MuseairHasher {
    #[inline(always)]
    fn finish(&self) -> u64 {
      self.0.finish()
    }

    #[inline(always)]
    fn write(&mut self, bytes: &[u8]) {
      self.0.write(bytes);
    }
  }
}

#[cfg(not(feature = "gxhash"))]
mod mix_hasher {
  use core::hash::Hasher;

  use super::mix64;

  /// Dependency-free streaming hasher based on mix64.
  /// 基于 mix64 的零依赖流式哈希器。
  #[derive(Clone, Debug, Default)]
  pub struct MixHasher(u64);

  impl MixHasher {
    /// Absorbs one 64-bit block into the state.
    /// 将一个 64 位块吸收进状态
    #[inline(always)]
    fn absorb(&mut self, v: u64) {
      self.0 = mix64(self.0 ^ v);
    }
  }

  impl Hasher for MixHasher {
    #[inline(always)]
    fn finish(&self) -> u64 {
      self.0
    }

    #[inline(always)]
    fn write(&mut self, bytes: &[u8]) {
      let mut chunks = bytes.chunks_exact(8);
      for chunk in &mut chunks {
        let mut b = [0u8; 8];
        b.copy_from_slice(chunk);
        self.absorb(u64::from_le_bytes(b));
      }
      let rem = chunks.remainder();
      if !rem.is_empty() {
        let mut b = [0u8; 8];
        b[..rem.len()].copy_from_slice(rem);
        // 尾部长度混入，避免不同长度输入产生相同哈希
        // Mix tail length in so inputs of different lengths never collide
        self.absorb(u64::from_le_bytes(b) ^ rem.len() as u64);
      }
    }
  }
}

const MIX_C1: u64 = 0xa076_1d64_78bd_642f;
const MIX_C2: u64 = 0xe703_7ed1_a0b4_28db;

/// 64-bit finalizer (wyhash-style 128-bit multiply with fold).
/// 64 位终结器（wyhash 风格 128 位乘法折叠）。
#[inline(always)]
pub const fn mix64(k: u64) -> u64 {
  let r = (k ^ MIX_C1) as u128 * (k ^ MIX_C2) as u128;
  (r as u64) ^ ((r >> 64) as u64)
}

/// Mixes key and seed into a guaranteed non-zero 64-bit hash.
/// 将键和种子混淆为保证非零的 64 位哈希值。
#[inline(always)]
pub const fn mix_key(key: u64, seed: u64) -> u64 {
  let hash = mix64(key.wrapping_add(seed));
  if hash == 0 { 1 } else { hash }
}

/// Fingerprint type constraint.
/// 指纹类型约束
pub trait Fingerprint:
  Default + Copy + PartialEq + BitXorAssign + BitXor<Output = Self> + 'static
{
  /// Computes a fingerprint from a hash.
  /// 从哈希值计算指纹
  fn from_hash(hash: u64) -> Self;
}

macro_rules! impl_fingerprint {
  ($ty:ty) => {
    impl Fingerprint for $ty {
      #[inline(always)]
      fn from_hash(hash: u64) -> Self {
        // mix64 已提供充分雪崩效应，直接使用低位即可
        // mix64 already provides sufficient avalanche effect, just use low bits
        hash as Self
      }
    }
  };
}

impl_fingerprint!(u8);
impl_fingerprint!(u16);
impl_fingerprint!(u32);
impl_fingerprint!(u64);
