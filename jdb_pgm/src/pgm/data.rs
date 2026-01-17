//! Pgm-Index with data ownership
//! 持有数据的 Pgm 索引

use std::{mem::size_of, ops::Deref};

use crate::{Key, Pgm};

/// Pgm-Index with data ownership
/// 持有数据的 Pgm 索引
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
#[derive(Clone, Debug)]
pub struct PgmData<K: Key> {
  pub pgm: Pgm<K>,
  pub sorted: Vec<K>,
}

impl<K: Key> Deref for PgmData<K> {
  type Target = Pgm<K>;

  #[inline]
  fn deref(&self) -> &Self::Target {
    &self.pgm
  }
}

impl<K: Key> PgmData<K> {
  /// Create Pgm-Index from sorted data slice
  /// 从已排序数据切片创建 Pgm 索引
  pub fn new(sorted: &[K], epsilon: usize) -> Self {
    let pgm = Pgm::new(sorted, epsilon);
    Self {
      pgm,
      sorted: sorted.to_vec(),
    }
  }

  /// Get reference to underlying data
  /// 获取底层数据引用
  #[inline]
  #[must_use]
  pub fn sorted(&self) -> &[K] {
    &self.sorted
  }

  /// Get position of key (None if absent)
  /// 获取键的位置（不存在则返回 None）
  #[inline]
  #[must_use]
  pub fn get(&self, key: K) -> Option<usize> {
    // SAFETY: Pgm::find_key callback is only called with indices within data bounds
    // (guaranteed by Pgm::predict_range).
    let idx = self
      .pgm
      .find_key(key, |i| unsafe { Some(*self.sorted.get_unchecked(i)) });
    // SAFETY: idx checked against len() before access
    if idx < self.sorted.len() && unsafe { *self.sorted.get_unchecked(idx) } == key {
      Some(idx)
    } else {
      None
    }
  }

  /// Batch lookup returning an iterator
  /// 批量查找（返回迭代器）
  #[inline]
  pub fn get_many<'a, I>(&'a self, keys: I) -> impl Iterator<Item = Option<usize>> + 'a
  where
    I: IntoIterator<Item = K> + 'a,
    <I as IntoIterator>::IntoIter: 'a,
  {
    keys.into_iter().map(move |k| self.get(k))
  }

  /// Count hits in batch
  /// 批量命中计数
  #[inline]
  pub fn count_hits<I>(&self, keys: I) -> usize
  where
    I: IntoIterator<Item = K>,
  {
    keys.into_iter().filter(|&k| self.get(k).is_some()).count()
  }

  /// Memory usage (including data)
  /// 内存占用（含数据）
  #[inline]
  #[must_use]
  pub fn memory_usage(&self) -> usize {
    self.sorted.len() * size_of::<K>() + self.pgm.mem_usage()
  }

  /// Get predicted position for a key (for benchmarking)
  /// 获取键的预测位置（用于基准测试）
  #[inline]
  #[must_use]
  pub fn predict_pos(&self, key: K) -> usize {
    self.pgm.predict(key)
  }

  /// Get statistics
  /// 获取统计信息
  #[inline]
  #[must_use]
  pub fn stats(&self) -> crate::PgmStats {
    crate::PgmStats {
      segments: self.pgm.segment_count(),
      avg_segment_size: self.pgm.avg_segment_size(),
      memory_bytes: self.memory_usage(),
    }
  }
}
