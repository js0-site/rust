//! Base generic Binary Fuse filter algorithm implementation.
//! 基础泛型 Binary Fuse 过滤器算法实现

pub mod build;
pub mod calc;
pub mod query;

use alloc::{boxed::Box, vec::Vec};
use core::{borrow::Borrow, hash::Hash};

#[cfg(feature = "bitcode")]
use bitcode::{Decode, Encode};

use crate::{Filter, hash::Fingerprint, prelude::bfuse::Desc};

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
  fn has<Q>(&self, key: &Q) -> bool
  where
    Q: ?Sized + Hash,
    u64: Borrow<Q>,
  {
    debug_assert_eq!(core::mem::size_of_val(key), core::mem::size_of::<u64>());
    let k = unsafe { *(key as *const Q as *const u64) };
    self.has_key(k)
  }

  #[inline(always)]
  fn len(&self) -> usize {
    self.fingerprints.len()
  }
}

impl<T: Fingerprint> Base<T> {
  /// Constructs the filter from a key iterator, returning Result.
  /// 从键迭代器构造过滤器，返回 Result。
  pub fn try_from_iterator<I>(keys: I) -> crate::Result<Self>
  where
    I: ExactSizeIterator<Item = u64> + Clone,
  {
    let (desc, fingerprints) = build::try_make(keys, 1000)?;
    Ok(Self { desc, fingerprints })
  }

  /// Constructs the filter from a slice of u64 keys, returning Result.
  /// 从 u64 键切片构造过滤器，返回 Result。
  pub fn try_from_slice(keys: &[u64]) -> crate::Result<Self> {
    Self::try_from_iterator(keys.iter().copied())
  }

  /// Constructs the filter from a Vec of u64 keys, returning Result.
  /// 从 u64 键 Vec 构造过滤器，返回 Result。
  pub fn try_from_vec(v: Vec<u64>) -> crate::Result<Self> {
    Self::try_from_iterator(v.iter().copied())
  }

  /// Constructs the filter from a key iterator.
  /// 从键迭代器构造过滤器。
  pub fn from_iterator<I>(keys: I) -> Self
  where
    I: ExactSizeIterator<Item = u64> + Clone,
  {
    let (desc, fingerprints) = build::make(keys, 1000);
    Self { desc, fingerprints }
  }

  /// Checks if a u64 key exists in the filter directly by value.
  /// 直接通过值检查 u64 键是否存在于过滤器中。
  #[inline(always)]
  pub fn has_key(&self, key: u64) -> bool {
    query::contains_impl(
      key,
      self.desc.seed,
      &self.fingerprints,
      self.desc.seg_len,
      self.desc.seg_len_mask,
      self.desc.seg_count_len,
    )
  }

  /// Batch checks membership for multiple u64 keys.
  /// 批量检查多个 u64 键的成员资格。
  #[inline]
  pub fn contains_batch(&self, keys: &[u64], results: &mut [bool]) {
    assert_eq!(keys.len(), results.len());
    let mut i = 0;
    while i + 1 < keys.len() {
      let k0 = unsafe { *keys.get_unchecked(i) };
      let k1 = unsafe { *keys.get_unchecked(i + 1) };
      unsafe {
        *results.get_unchecked_mut(i) = self.has_key(k0);
        *results.get_unchecked_mut(i + 1) = self.has_key(k1);
      }
      i += 2;
    }
    if i < keys.len() {
      unsafe {
        *results.get_unchecked_mut(i) = self.has_key(*keys.get_unchecked(i));
      }
    }
  }
}

impl<T: Fingerprint> From<&[u64]> for Base<T> {
  fn from(keys: &[u64]) -> Self {
    Self::from_iterator(keys.iter().copied())
  }
}

impl<T: Fingerprint> From<&Vec<u64>> for Base<T> {
  fn from(v: &Vec<u64>) -> Self {
    Self::from(v.as_slice())
  }
}

impl<T: Fingerprint> From<Vec<u64>> for Base<T> {
  fn from(v: Vec<u64>) -> Self {
    Self::from(v.as_slice())
  }
}

/// A 8-bit Binary Fuse filter using the base implementation.
pub type Bf8 = Base<u8>;

/// A 16-bit Binary Fuse filter using the base implementation.
pub type Bf16 = Base<u16>;

/// A 32-bit Binary Fuse filter using the base implementation.
pub type Bf32 = Base<u32>;
