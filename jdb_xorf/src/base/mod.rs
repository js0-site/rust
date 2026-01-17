//! Base generic Binary Fuse filter algorithm implementation.
//! 基础泛型 Binary Fuse 过滤器算法实现

pub mod build;
pub mod calc;
pub mod query;

use alloc::{boxed::Box, vec::Vec};
use core::borrow::Borrow;
use core::hash::Hash;

#[cfg(feature = "bitcode")]
use bitcode::{Decode, Encode};

use crate::{
  Filter,
  hash::Fingerprint,
  prelude::bfuse::Desc,
};

// Re-export specific internal functions if necessary, mostly for internal use.


/// Generic Binary Fuse filter.
/// 泛型 Binary Fuse 过滤器
#[cfg_attr(feature = "bitcode", derive(Decode, Encode))]
#[derive(Debug, Clone)]
pub struct Base<T> {
  /// Descriptor
  /// 描述符
  pub desc: Desc,
  /// Fingerprint data
  /// 指纹数据
  pub fingerprints: Box<[T]>,
}

impl<T: Fingerprint> Filter<u64> for Base<T> {
  #[inline(always)]
  fn has<Q: ?Sized>(&self, key: &Q) -> bool
  where
    u64: Borrow<Q>,
    Q: Hash,
  {
    let k = unsafe { *(key as *const Q as *const u64) };
    query::contains_impl(
      &k,
      self.desc.seed,
      &self.fingerprints,
      self.desc.seg_len,
      self.desc.seg_len_mask,
      self.desc.seg_count_len,
    )
  }

  #[inline(always)]
  fn len(&self) -> usize {
    self.fingerprints.len()
  }
}

impl<T: Fingerprint> Base<T> {
  /// Constructs the filter from a key iterator.
  /// 从键迭代器构造过滤器。
  pub fn from_iterator<I>(keys: I) -> Self
  where
    I: ExactSizeIterator<Item = u64> + Clone,
  {
    let (desc, fingerprints) = build::make(keys, 1000);
    Self { desc, fingerprints }
  }
}

impl<T: Fingerprint> From<&[u64]> for Base<T> {
  fn from(keys: &[u64]) -> Self {
    Self::from_iterator(keys.iter().copied())
  }
}

impl<T: Fingerprint> From<&Vec<u64>> for Base<T> {
  fn from(v: &Vec<u64>) -> Self {
    Self::from_iterator(v.iter().copied())
  }
}

impl<T: Fingerprint> From<Vec<u64>> for Base<T> {
  fn from(v: Vec<u64>) -> Self {
    Self::from_iterator(v.iter().copied())
  }
}

/// A 8-bit Binary Fuse filter using the base implementation.
pub type Bf8 = Base<u8>;

/// A 16-bit Binary Fuse filter using the base implementation.
pub type Bf16 = Base<u16>;

/// A 32-bit Binary Fuse filter using the base implementation.
pub type Bf32 = Base<u32>;
