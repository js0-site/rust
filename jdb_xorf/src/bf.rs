//! Implements a Bfer abstraction for constructing filters from arbitrary types.

use alloc::{boxed::Box, vec::Vec};
use core::{
  borrow::Borrow,
  hash::{Hash, Hasher},
  marker::PhantomData,
};

#[cfg(feature = "bitcode")]
use bitcode::{Decode, Encode};

use crate::{Filter, RapidHasher};

/// Bfer for creating and querying filters with arbitrary key types.
/// 用于构建和查询任意键类型的过滤器的构建器。
///
/// A `Bf` wraps an underlying `Filter<u64>` construction and provides automatic
/// hashing and deduplication for arbitrary keys (e.g., `String`, `&[u8]`).
///
/// `Bf` 包装了底层的 `Filter<u64>` 构造，并为任意键（例如 `String`、`&[u8]`）提供自动哈希和去重。
///
/// ```
/// # extern crate alloc;
/// use jdb_xorf::{Filter, Bf, Bf8};
/// # use alloc::vec::Vec;
/// # use rand::distr::Alphanumeric;
/// # use rand::Rng;
///
/// const SAMPLE_SIZE: usize = 1_000_000;
/// let passwords: Vec<String> = (0..SAMPLE_SIZE)
///     .map(|_| rand::rng().sample_iter(&Alphanumeric).take(30).map(char::from).collect())
///     .collect();
///
/// // Bf enables safe construction from arbitrary types with auto-deduplication.
/// let pw_filter: Bf<String, Bf8> = Bf::from(&passwords);
///
/// for password in passwords {
///     assert!(pw_filter.has(&password));
/// }
/// ```
#[cfg_attr(feature = "bitcode", derive(Decode, Encode))]
pub struct Bf<T: ?Sized, F, H = RapidHasher> {
  /// The underlying filter
  /// 底层过滤器
  pub filter: F,
  /// Marker for the hasher and key types
  /// 哈希器和键类型标记
  pub _phantom: PhantomData<(H, Box<T>)>,
}

/// Computes the hash value for a key.
/// 计算键的哈希值。
#[inline(always)]
fn hash<H: Hasher + Default, T: Hash + ?Sized>(key: &T) -> u64 {
  let mut hasher = H::default();
  key.hash(&mut hasher);
  hasher.finish()
}

impl<T: ?Sized, F, H> Filter<T> for Bf<T, F, H>
where
  T: Hash,
  H: Hasher + Default,
  F: Filter<u64>,
{
  fn has<Q: ?Sized>(&self, key: &Q) -> bool
  where
    T: Borrow<Q>,
    Q: Hash,
  {
    self.filter.has(&hash::<H, Q>(key))
  }

  fn len(&self) -> usize {
    self.filter.len()
  }
}

impl<T, F, H> Bf<T, F, H>
where
  T: Hash,
  H: Hasher + Default,
  F: Filter<u64>,
{
  /// Construct a Bf from a slice of keys.
  /// 从键切片构造 Bf。
  ///
  /// Automatically handles hashing, sorting, and deduplication to ensure
  /// filter construction succeeds.
  /// 自动处理哈希、排序和去重，以确保过滤器构造成功。
  ///
  /// # Panics
  /// Only in the extremely unlikely event that the underlying filter fails to build even after deduplication.
  /// 仅在底层过滤器即使去重后仍极其不可能构建失败的情况下。
  pub fn from(keys: &[T]) -> Self
  where
    F: From<Vec<u64>>,
  {
    let mut keys: Vec<u64> = keys.iter().map(hash::<H, T>).collect();
    keys.sort_unstable();
    keys.dedup();
    Self {
      filter: F::from(keys),
      _phantom: PhantomData,
    }
  }
}

impl<T, F, H> From<&Vec<T>> for Bf<T, F, H>
where
  T: Hash,
  H: Hasher + Default,
  F: Filter<u64> + From<Vec<u64>>,
{
  fn from(v: &Vec<T>) -> Self {
    Self::from(v.as_slice())
  }
}

impl<T, F, H> From<&[T]> for Bf<T, F, H>
where
  T: Hash,
  H: Hasher + Default,
  F: Filter<u64> + From<Vec<u64>>,
{
  fn from(keys: &[T]) -> Self {
    Self::from(keys)
  }
}

impl<T: ?Sized, F, H> Bf<T, F, H>
where
  T: Hash,
  H: Hasher + Default,
  F: Filter<u64>,
{
  /// Wraps an existing filter.
  /// 包装现有的过滤器。
  ///
  /// This is useful when you have a pre-built or deserialized filter (e.g. `Bf8`)
  /// and want to use it with typed keys (e.g. `String`).
  /// 当你有一个预构建或反序列化的过滤器（例如 `Bf8`）并希望将其与类型化键（例如 `String`）一起使用时，这很有用。
  ///
  /// # Safety
  /// The caller must ensure that the underlying filter was constructed using keys hashed with `H`.
  /// 调用者必须确底层过滤器是使用 `H` 哈希过的键构建的。
  /// Returns `true` if the underlying filter contains the specified key.
  /// 如果底层过滤器包含指定的键，则返回 `true`。
  ///
  /// Allows querying with borrowed types (e.g., `&str` for `String` keys).
  /// 允许使用借用类型进行查询（例如 `String` 键使用 `&str`）。
  pub fn has<Q>(&self, key: &Q) -> bool
  where
    T: Borrow<Q>,
    Q: Hash + ?Sized,
  {
    let borrowed: &Q = key.borrow();
    self.filter.has(&hash::<H, Q>(borrowed))
  }

  /// Wraps an existing filter.
  /// 包装现有的过滤器。
  ///
  /// This is useful when you have a pre-built or deserialized filter (e.g. `Bf8`)
  /// and want to use it with typed keys (e.g. `String`).
  /// 当你有一个预构建或反序列化的过滤器（例如 `Bf8`）并希望将其与类型化键（例如 `String`）一起使用时，这很有用。
  ///
  /// # Safety
  /// The caller must ensure that the underlying filter was constructed using keys hashed with `H`.
  /// 调用者必须确底层过滤器是使用 `H` 哈希过的键构建的。
  pub fn wrap(filter: F) -> Self {
    Self {
      filter,
      _phantom: PhantomData,
    }
  }
}

impl<T: ?Sized, F, H> From<Bf<&T, F, H>> for Bf<T, F, H> {
  fn from(src: Bf<&T, F, H>) -> Self {
    Self {
      filter: src.filter,
      _phantom: PhantomData,
    }
  }
}
