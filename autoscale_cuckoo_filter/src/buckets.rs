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
    let buckets = Self::required_buckets(buckets_hint);
    let idx_bits = buckets.trailing_zeros() as usize;
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
    hint.max(1).next_power_of_two()
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

  /// Extract fingerprint from hash (fp_bits ∈ [1, 56]).
  /// 从哈希提取指纹（fp_bits ∈ [1, 56]）
  #[inline(always)]
  pub fn fingerprint(&self, hash: u64) -> u64 {
    hash >> (64 - self.fp_bits)
  }

  /// Extract fingerprint and compute bucket index simultaneously from hash.
  /// 同时从哈希提取指纹并计算桶索引（消除重复计算）
  #[inline(always)]
  pub fn fp_and_index(&self, hash: u64) -> (u64, usize) {
    (self.fingerprint(hash), self.index(hash))
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

  /// Check if bucket contains fingerprint.
  /// 检查桶是否包含指纹
  #[inline]
  pub fn contains(&self, idx: usize, fp: u64) -> bool {
    debug_assert_ne!(fp, 0);
    self.find_offset(idx, fp).is_some()
  }

  /// Find bit offset of the first entry equal to `target` in bucket `idx`.
  /// 查找桶中第一个等于 target 的条目的位偏移
  #[inline(always)]
  fn find_offset(&self, idx: usize, target: u64) -> Option<usize> {
    let base = self.bucket_bits * idx;
    let fp_bits = self.fp_bits;
    let fp_mask = self.fp_mask;

    // Fast path: one u64 load covers the whole bucket (e.g. 4×16, 2×32 bits)
    // 快速路径：一次 u64 读取覆盖整个桶（如 4×16、2×32 位）
    if self.bucket_bits <= 64 {
      let bucket = self.bits.read_raw(base);
      // Branchless scan: OR-combine match flags, then locate via trailing_zeros
      // 无分支扫描：按位或合并匹配标志，再用 trailing_zeros 定位
      let mut hits = 0u64;
      for i in 0..self.entries {
        hits |= (((bucket >> (fp_bits * i)) & fp_mask == target) as u64) << i;
      }
      if hits == 0 {
        return None;
      }
      return Some(base + fp_bits * hits.trailing_zeros() as usize);
    }

    // Fallback for larger buckets
    // 更大桶的回退路径
    for i in 0..self.entries {
      let off = base + fp_bits * i;
      if self.bits.get_uint_masked(off, fp_mask) == target {
        return Some(off);
      }
    }
    None
  }

  /// Replace first entry matching `target` with `replacement`.
  /// 将第一个匹配 target 的条目替换为 replacement
  #[inline]
  fn find_and_replace(&mut self, idx: usize, target: u64, replacement: u64) -> bool {
    match self.find_offset(idx, target) {
      Some(off) => {
        self.bits.set_uint_masked(off, self.fp_mask, replacement);
        true
      }
      None => false,
    }
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
      // Fast check: when starting a new bucket that fits in one u64,
      // skip entire empty bucket at once.
      // 快速检查：当桶可容纳于单个 u64 时，单次读取跳过全空桶
      if self.entry_i == 0 && self.buckets.bucket_bits <= 64 {
        let base = self.buckets.bucket_bits * self.bucket_i;
        let bmask = (1u64 << self.buckets.bucket_bits) - 1;
        if (self.buckets.bits.read_raw(base) & bmask) == 0 {
          self.bucket_i += 1;
          continue;
        }
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
