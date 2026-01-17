#![cfg_attr(docsrs, feature(doc_cfg))]

//! Shared prefix length calculation
//! 共享前缀长度计算

/// Find shared prefix length
/// 查找共享前缀长度
#[inline]
pub fn shared_prefix_len(a: &[u8], b: &[u8]) -> usize {
  let len = a.len().min(b.len());
  let mut prefix = 0;

  // Compare u64 at a time using chunks (efficient on 64-bit)
  // 使用 chunks 一次比较 8 字节（64位系统高效）
  type Chunk = u64;

  for (ac, bc) in a
    .chunks_exact(size_of::<Chunk>())
    .zip(b.chunks_exact(size_of::<Chunk>()))
  {
    // Use native endian load for speed, handle endianness only on mismatch
    // 使用本机字节序加载以提升速度，仅在不匹配时处理字节序
    let av = Chunk::from_ne_bytes(ac.try_into().unwrap());
    let bv = Chunk::from_ne_bytes(bc.try_into().unwrap());
    let xor = av ^ bv;
    if xor != 0 {
      // On Little Endian, trailing_zeros / 8 is the index. On Big Endian, leading_zeros / 8.
      // 小端序用 trailing_zeros / 8，大端序用 leading_zeros / 8
      #[cfg(target_endian = "little")]
      return prefix + (xor.trailing_zeros() as usize >> 3);
      #[cfg(target_endian = "big")]
      return prefix + (xor.leading_zeros() as usize >> 3);
    }
    prefix += size_of::<Chunk>();
  }

  // Compare remaining bytes
  // 比较剩余字节
  while prefix < len && a[prefix] == b[prefix] {
    prefix += 1;
  }
  prefix
}
