/// GXHash finalization mix implementation.
///
/// GXHash is a fast non-cryptographic hash algorithm that provides good avalanche properties.
/// This implementation provides a similar interface to MurmurHash3's mix64 for easy swapping.
///
/// Reference: https://github.com/kamranahmedse/gxhash
pub const fn mix64(k: u64) -> u64 {
  // GXHash finalization - provides avalanche effect similar to MurmurHash3
  let mut h = k;
  h ^= h >> 33;
  h = h.wrapping_mul(0xff51afd7ed558ccd);
  h ^= h >> 33;
  h = h.wrapping_mul(0xc4ceb9fe1a85ec53);
  h ^= h >> 33;
  h
}
