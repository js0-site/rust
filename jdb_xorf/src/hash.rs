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
