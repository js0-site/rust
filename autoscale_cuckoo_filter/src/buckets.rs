//! Bucket storage for cuckoo filter fingerprints.
//! 布谷鸟过滤器指纹的桶存储

use std::hash::Hasher;

use crate::bits::Bits;

/// Bucket array for storing fingerprints.
/// 用于存储指纹的桶数组
#[derive(Debug, Clone, bitcode::Encode, bitcode::Decode)]
pub struct Buckets {
  fp_bits: usize,
  fp_mask: u64,
  entries: usize,
  bucket_bits: usize,
  idx_mask: usize,
  bits: Bits,
}

impl Buckets {
  /// Create new bucket array.
  /// 创建新的桶数组
  pub fn new(fp_bits: usize, entries: usize, buckets_hint: usize) -> Self {
    let idx_bits = buckets_hint.next_power_of_two().trailing_zeros() as usize;
    let bucket_bits = fp_bits * entries;
    let bits = Bits::new(bucket_bits << idx_bits);
    let idx_mask = (1 << idx_bits) - 1;
    let fp_mask = (1u64 << fp_bits) - 1;
    Buckets {
      fp_bits,
      fp_mask,
      entries,
      bucket_bits,
      idx_mask,
      bits,
    }
  }

  /// Calculate required number of buckets.
  /// 计算所需的桶数量
  #[inline]
  pub fn required_buckets(hint: usize) -> usize {
    hint.next_power_of_two()
  }

  /// Returns number of buckets.
  /// 返回桶数量
  #[inline]
  pub fn len(&self) -> usize {
    self.idx_mask + 1
  }

  /// Returns total entry count.
  /// 返回总条目数
  #[inline]
  pub fn total_entries(&self) -> usize {
    self.len() * self.entries
  }

  /// Returns bits used.
  /// 返回使用的位数
  #[inline]
  pub fn bits(&self) -> u64 {
    self.bits.len() as u64
  }

  /// Compute bucket index from hash.
  /// 从哈希计算桶索引
  #[inline(always)]
  pub fn index(&self, hash: u64) -> usize {
    (hash as usize) & self.idx_mask
  }

  /// Compute alternate bucket index for a given bucket index and fingerprint.
  /// 为给定的桶索引和指纹计算备选桶索引
  #[inline(always)]
  pub fn alt_index<H: Hasher + Clone>(&self, hasher: &H, i: usize, fp: u64) -> usize {
    self.index(i as u64 ^ crate::hash(hasher, &fp))
  }

  /// Extract fingerprint from hash.
  /// 从哈希提取指纹
  #[inline]
  pub fn fingerprint(&self, hash: u64) -> u64 {
    hash >> (64 - self.fp_bits)
  }

  /// Returns entries per bucket.
  /// 返回每桶条目数
  #[inline]
  pub fn entries_per_bucket(&self) -> usize {
    self.entries
  }

  /// Returns fingerprint bit width.
  /// 返回指纹位宽
  #[inline]
  pub fn fp_bitwidth(&self) -> usize {
    self.fp_bits
  }

  /// Returns iterator over non-zero fingerprints.
  /// 返回非零指纹的迭代器
  #[inline]
  pub fn iter(&self) -> Iter<'_> {
    Iter::new(self)
  }

  /// Check if bucket contains fingerprint (optimized for 4 entries).
  /// 检查桶是否包含指纹（针对 4 条目优化）
  #[inline]
  pub fn contains(&self, idx: usize, fp: u64) -> bool {
    debug_assert_ne!(fp, 0);
    let base = self.bucket_bits * idx;
    let fp_bits = self.fp_bits;
    let fp_mask = self.fp_mask;

    // For small fp_bits (<=16) and 4 entries, read entire bucket at once (4*16=64 bits max)
    // 对于小指纹位数（<=16）且 4 条目，一次读取整个桶（最多 4*16=64 位）
    if self.entries == 4 && fp_bits <= 16 {
      let bucket = self.bits.read_raw(base);
      // Use | instead of || to avoid branch misprediction
      // 使用 | 代替 || 避免分支预测失败
      return ((bucket & fp_mask) == fp)
        | (((bucket >> fp_bits) & fp_mask) == fp)
        | (((bucket >> (fp_bits * 2)) & fp_mask) == fp)
        | (((bucket >> (fp_bits * 3)) & fp_mask) == fp);
    }

    // Fallback for larger fp_bits
    // 大指纹位数的回退路径
    for i in 0..self.entries {
      let off = base + fp_bits * i;
      if self.bits.get_uint_masked(off, fp_mask) == fp {
        return true;
      }
    }
    false
  }

  /// Find first entry matching `target` and replace it with `replacement`.
  /// 查找第一个与 `target` 匹配的条目并将其替换为 `replacement`
  #[inline]
  fn find_and_replace(&mut self, idx: usize, target: u64, replacement: u64) -> bool {
    let base = self.bucket_bits * idx;
    let fp_bits = self.fp_bits;
    let fp_mask = self.fp_mask;

    // For small fp_bits (<=16) and 4 entries, read entire bucket at once
    // 对于小指纹位数（<=16）且 4 条目，一次读取整个桶
    if self.entries == 4 && fp_bits <= 16 {
      let (raw, byte_idx, bit_off) = self.bits.read_raw_parts(base);
      let bucket = raw >> bit_off;
      if (bucket & fp_mask) == target {
        let new_raw = (raw & !(fp_mask << bit_off)) | (replacement << bit_off);
        self.bits.write_raw_at_byte(byte_idx, new_raw);
        return true;
      }
      let off1 = bit_off + fp_bits;
      if ((bucket >> fp_bits) & fp_mask) == target {
        let new_raw = (raw & !(fp_mask << off1)) | (replacement << off1);
        self.bits.write_raw_at_byte(byte_idx, new_raw);
        return true;
      }
      let off2 = off1 + fp_bits;
      if ((bucket >> (fp_bits * 2)) & fp_mask) == target {
        let new_raw = (raw & !(fp_mask << off2)) | (replacement << off2);
        self.bits.write_raw_at_byte(byte_idx, new_raw);
        return true;
      }
      let off3 = off2 + fp_bits;
      if ((bucket >> (fp_bits * 3)) & fp_mask) == target {
        let new_raw = (raw & !(fp_mask << off3)) | (replacement << off3);
        self.bits.write_raw_at_byte(byte_idx, new_raw);
        return true;
      }
      return false;
    }

    // Fallback for other configurations
    // 其他配置的回退路径
    for i in 0..self.entries {
      let off = base + fp_bits * i;
      if self.bits.get_uint_masked(off, fp_mask) == target {
        self.bits.set_uint_masked(off, fp_mask, replacement);
        return true;
      }
    }
    false
  }

  /// Try to insert fingerprint into bucket.
  /// 尝试将指纹插入桶
  #[inline(always)]
  pub fn try_insert(&mut self, idx: usize, fp: u64) -> bool {
    debug_assert_ne!(fp, 0);
    self.find_and_replace(idx, 0, fp)
  }

  /// Swap fingerprint with random entry in bucket using fastrand.
  /// 使用 fastrand 与桶中随机条目交换指纹
  #[inline]
  pub fn random_swap(&mut self, idx: usize, fp: u64) -> u64 {
    let i = fastrand::usize(0..self.entries);
    let off = self.bucket_bits * idx + self.fp_bits * i;
    let old = self.bits.get_uint_masked(off, self.fp_mask);
    self.bits.set_uint_masked(off, self.fp_mask, fp);
    debug_assert_ne!(fp, 0);
    debug_assert_ne!(old, 0);
    old
  }

  /// Remove fingerprint from bucket.
  /// 从桶中移除指纹
  #[inline(always)]
  pub fn remove(&mut self, idx: usize, fp: u64) -> bool {
    debug_assert_ne!(fp, 0);
    self.find_and_replace(idx, fp, 0)
  }

  /// Get fingerprint at specific position.
  /// 获取指定位置的指纹
  #[inline]
  fn get_fp(&self, bucket_idx: usize, entry_idx: usize) -> u64 {
    let off = self.bucket_bits * bucket_idx + self.fp_bits * entry_idx;
    self.bits.get_uint_masked(off, self.fp_mask)
  }
}

/// Iterator over bucket fingerprints.
/// 桶指纹迭代器
#[derive(Debug)]
pub struct Iter<'a> {
  buckets: &'a Buckets,
  bucket_i: usize,
  entry_i: usize,
}

impl<'a> Iter<'a> {
  fn new(buckets: &'a Buckets) -> Self {
    Iter {
      buckets,
      bucket_i: 0,
      entry_i: 0,
    }
  }
}

impl Iterator for Iter<'_> {
  type Item = (usize, u64);

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      if self.bucket_i == self.buckets.len() {
        return None;
      }
      if self.entry_i == self.buckets.entries {
        self.bucket_i += 1;
        self.entry_i = 0;
        continue;
      }
      let fp = self.buckets.get_fp(self.bucket_i, self.entry_i);
      self.entry_i += 1;
      if fp != 0 {
        return Some((self.bucket_i, fp));
      }
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn basic_ops() {
    let mut buckets = Buckets::new(8, 4, 1000);
    assert_eq!(buckets.len(), 1024);
    assert_eq!(buckets.bits(), 1024 * 8 * 4);

    for i in 0..4 {
      assert!(!buckets.contains(333, 100 + i));
      assert!(buckets.try_insert(333, 100 + i));
      assert!(buckets.contains(333, 100 + i));
    }
    assert!(!buckets.try_insert(333, 104));

    let old = buckets.random_swap(333, 104);
    assert!(buckets.contains(333, 104));
    assert!(!buckets.contains(333, old));
  }

  #[test]
  fn iter_skips_zero() {
    let mut buckets = Buckets::new(8, 4, 1);
    assert!(buckets.try_insert(0, 10));
    assert!(buckets.try_insert(0, 11));
    assert!(buckets.remove(0, 10));

    let collected: Vec<_> = buckets.iter().collect();
    assert_eq!(collected, vec![(0, 11)]);
  }
}
