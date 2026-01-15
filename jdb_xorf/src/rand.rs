//! Random number generation for xor filters.
//! xor filter 的随机数生成
//!
//! This module uses RapidHash finalization mix for fast non-cryptographic hashing.
//! 本模块使用 RapidHash 终结混淆进行快速非加密哈希

/// Applies a finalization mix to a randomly-seeded key, resulting in an avalanched hash.
/// 对随机种子密钥应用终结混淆，产生雪崩哈希
/// This helps avoid high false-positive ratios (see Section 4 in the paper).
/// 这有助于避免高误报率（参见论文第 4 节）
///
/// This uses RapidHash's finalization mix.
/// 使用 RapidHash 的终结混淆
#[inline(always)]
pub const fn mix(key: u64, seed: u64) -> u64 {
  crate::rapidhash::mix64(key.overflowing_add(seed).0)
}
