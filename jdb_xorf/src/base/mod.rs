//! Base generic Binary Fuse filter algorithm implementation.
//! 基础泛型 Binary Fuse 过滤器算法实现

pub mod build;
pub mod calc;
pub mod query;

use alloc::{boxed::Box, vec::Vec};
use core::{
  borrow::Borrow,
  hash::Hash,
  iter::FromIterator,
  mem::{size_of, size_of_val},
  ptr::read_unaligned,
};

#[cfg(feature = "bitcode")]
use bitcode::{Decode, Encode};

use crate::{Desc, Filter, Fingerprint};

// Re-export specific internal functions if necessary, mostly for internal use.

/// Generic Binary Fuse filter.
/// 泛型 Binary Fuse 过滤器
#[cfg_attr(feature = "bitcode", derive(Decode, Encode))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Base<T> {
  /// Descriptor
  /// 描述符
  pub desc: Desc,
  /// Fingerprint data
  /// 指纹数据
  pub fingerprints: Box<[T]>,
}

impl<T: Fingerprint> Default for Base<T> {
  #[inline]
  fn default() -> Self {
    Self {
      desc: Desc::default(),
      fingerprints: Box::new([]),
    }
  }
}

impl<T: Fingerprint> Filter<u64> for Base<T> {
  #[inline(always)]
  fn has<Q>(&self, key: &Q) -> bool
  where
    Q: ?Sized + Hash,
    u64: Borrow<Q>,
  {
    if size_of_val(key) == size_of::<u64>() {
      // SAFETY: Under Rust coherence rules, the only type `Q` implementing `u64: Borrow<Q>` is `Q = u64`.
      // `read_unaligned` safely converts the referenced 8 bytes to `u64`.
      // SAFETY: 根据 Rust 孤儿规则，实现 `u64: Borrow<Q>` 的类型 `Q` 仅能是 `u64`。
      // `read_unaligned` 安全地将引用的 8 字节读取为 `u64`。
      let k = unsafe { read_unaligned(key as *const Q as *const u64) };
      self.has_key(k)
    } else {
      false
    }
  }

  #[inline(always)]
  fn has_key(&self, key: u64) -> bool {
    Base::has_key(self, key)
  }

  #[inline(always)]
  fn contains_batch_keys(&self, keys: &[u64], results: &mut [bool]) {
    self.contains_batch(keys, results);
  }

  #[inline(always)]
  fn len(&self) -> usize {
    self.fingerprints.len()
  }

  #[inline(always)]
  fn bytes(&self) -> usize {
    Base::bytes(self)
  }

  #[inline(always)]
  fn bits(&self) -> usize {
    Base::bits(self)
  }
}

impl<T: Fingerprint> Base<T> {
  /// Returns the total memory footprint in bytes.
  /// 返回该过滤器的总内存占用（字节数）。
  #[inline(always)]
  pub fn bytes(&self) -> usize {
    size_of::<Desc>() + self.fingerprints.len() * size_of::<T>()
  }

  /// Returns the total memory footprint in bits.
  /// 返回该过滤器的总内存占用（位数）。
  #[inline(always)]
  pub fn bits(&self) -> usize {
    self.bytes().saturating_mul(8)
  }

  /// Validates internal descriptor and fingerprint array invariants.
  /// 校验内部描述符与指纹数组的不变式约束。
  #[inline]
  pub fn is_valid(&self) -> bool {
    if self.fingerprints.is_empty() {
      self.desc.seg_count_len == 0 || self.desc == Desc::default()
    } else {
      self.desc.validate(self.fingerprints.len())
    }
  }

  /// Constructs the filter from a key iterator, returning Result.
  /// 从键迭代器构造过滤器，返回 Result。
  pub fn try_from_iterator<I>(keys: I) -> crate::Result<Self>
  where
    I: IntoIterator<Item = u64>,
  {
    let keys: Vec<u64> = keys.into_iter().collect();
    Self::try_from_vec(keys)
  }

  /// Constructs the filter from a slice of u64 keys, returning Result.
  /// 从 u64 键切片构造过滤器，返回 Result。
  ///
  /// Accepts `Vec<u64>`, `&Vec<u64>`, `&[u64]`, arrays, etc. via `AsRef`.
  /// 通过 `AsRef` 接受 `Vec<u64>`、`&Vec<u64>`、`&[u64]`、数组等。
  pub fn try_from_slice(keys: impl AsRef<[u64]>) -> crate::Result<Self> {
    let (desc, fingerprints) = build::try_make(keys.as_ref(), 1000)?;
    Ok(Self { desc, fingerprints })
  }

  /// Constructs the filter from a Vec of u64 keys, returning Result.
  /// 从 u64 键 Vec 构造过滤器，返回 Result。
  pub fn try_from_vec(v: Vec<u64>) -> crate::Result<Self> {
    Self::try_from_slice(&v)
  }

  /// Constructs the filter from a key iterator.
  /// 从键迭代器构造过滤器。
  pub fn from_iterator<I>(keys: I) -> Self
  where
    I: IntoIterator<Item = u64>,
  {
    let keys: Vec<u64> = keys.into_iter().collect();
    Self::from(keys.as_slice())
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
  #[inline(always)]
  pub fn contains_batch(&self, keys: &[u64], results: &mut [bool]) {
    debug_assert_eq!(keys.len(), results.len());
    query::contains_batch_impl(
      keys,
      results,
      self.desc.seed,
      &self.fingerprints,
      self.desc.seg_len,
      self.desc.seg_len_mask,
      self.desc.seg_count_len,
    );
  }

  /// Batch checks membership for multiple u64 keys, returning a Vec of booleans.
  /// 批量检查多个 u64 键的成员资格，返回布尔值 Vec。
  #[inline]
  pub fn contains_batch_vec(&self, keys: &[u64]) -> Vec<bool> {
    crate::batch_vec(keys.len(), |results| self.contains_batch(keys, results))
  }
}

impl<T: Fingerprint> FromIterator<u64> for Base<T> {
  fn from_iter<I: IntoIterator<Item = u64>>(iter: I) -> Self {
    let v: Vec<u64> = iter.into_iter().collect();
    Self::from(v.as_slice())
  }
}

impl<T: Fingerprint> From<&[u64]> for Base<T> {
  fn from(keys: &[u64]) -> Self {
    let (desc, fingerprints) = build::make(keys, 1000);
    Self { desc, fingerprints }
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

impl<T: Fingerprint, const N: usize> From<[u64; N]> for Base<T> {
  fn from(arr: [u64; N]) -> Self {
    Self::from(&arr[..])
  }
}

impl<T: Fingerprint, const N: usize> From<&[u64; N]> for Base<T> {
  fn from(arr: &[u64; N]) -> Self {
    Self::from(&arr[..])
  }
}

/// A 8-bit Binary Fuse filter using the base implementation.
pub type Bf8 = Base<u8>;

/// A 16-bit Binary Fuse filter using the base implementation.
pub type Bf16 = Base<u16>;

/// A 32-bit Binary Fuse filter using the base implementation.
pub type Bf32 = Base<u32>;

/// A 64-bit Binary Fuse filter using the base implementation.
pub type Bf64 = Base<u64>;
