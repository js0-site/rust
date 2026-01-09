//! Random number generation for xor filters.
//!
//! This module uses GXHash finalization mix for fast non-cryptographic hashing.

/// Applies a finalization mix to a randomly-seeded key, resulting in an avalanched hash.
/// This helps avoid high false-positive ratios (see Section 4 in the paper).
///
/// This uses GXHash's finalization mix.
#[inline]
pub const fn mix(key: u64, seed: u64) -> u64 {
  crate::gxhash::mix64(key.overflowing_add(seed).0)
}
