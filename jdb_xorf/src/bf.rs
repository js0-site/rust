//! Implements a Bfer abstraction for constructing filters from arbitrary types.

use alloc::{boxed::Box, string::String, vec::Vec};
use core::{
  borrow::Borrow,
  fmt::{Debug, Formatter, Result as FmtResult},
  hash::{Hash, Hasher},
  iter::FromIterator,
  marker::PhantomData,
};

#[cfg(feature = "bitcode")]
use bitcode::{Decode, Encode};

use crate::{Base, DefaultHasher, Filter, Fingerprint};

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
/// # use rand::{Rng, RngExt};
///
/// const SAMPLE_SIZE: usize = 100;
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
pub struct Bf<T: ?Sized, F, H = DefaultHasher> {
  /// The underlying filter
  /// 底层过滤器
  pub filter: F,
  /// Marker for the hasher and key types
  /// 哈希器和键类型标记
  pub _phantom: PhantomData<(H, Box<T>)>,
}

impl<T: ?Sized, F: Clone, H> Clone for Bf<T, F, H> {
  #[inline]
  fn clone(&self) -> Self {
    Self {
      filter: self.filter.clone(),
      _phantom: PhantomData,
    }
  }
}

impl<T: ?Sized, F: Debug, H> Debug for Bf<T, F, H> {
  fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
    f.debug_struct("Bf").field("filter", &self.filter).finish()
  }
}

impl<T: ?Sized, F: PartialEq, H> PartialEq for Bf<T, F, H> {
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    self.filter == other.filter
  }
}

impl<T: ?Sized, F: Eq, H> Eq for Bf<T, F, H> {}

impl<T: ?Sized, F: Default, H> Default for Bf<T, F, H> {
  #[inline]
  fn default() -> Self {
    Self {
      filter: F::default(),
      _phantom: PhantomData,
    }
  }
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
  H: Hasher + Default,
  F: Filter<u64>,
{
  #[inline(always)]
  fn has<Q>(&self, key: &Q) -> bool
  where
    Q: ?Sized + Hash,
    T: Borrow<Q>,
  {
    self.filter.has(&hash::<H, Q>(key))
  }

  #[inline(always)]
  fn len(&self) -> usize {
    self.filter.len()
  }

  #[inline(always)]
  fn bytes(&self) -> usize {
    self.filter.bytes()
  }

  #[inline(always)]
  fn bits(&self) -> usize {
    self.filter.bits()
  }
}

/// Sorts and deduplicates a hash vector in place.
/// 原地排序并去重哈希向量。
#[inline]
fn sort_dedup(v: &mut Vec<u64>) {
  v.sort_unstable();
  v.dedup();
}

/// Hashes, sorts, and deduplicates a slice of keys.
/// 对键切片进行哈希、排序和去重。
#[inline]
fn hash_sort_dedup<H: Hasher + Default, T: Hash>(keys: &[T]) -> Vec<u64> {
  let mut hashes: Vec<u64> = keys.iter().map(hash::<H, T>).collect();
  sort_dedup(&mut hashes);
  hashes
}

impl<T, U, H> Bf<T, Base<U>, H>
where
  T: Hash,
  U: Fingerprint,
  H: Hasher + Default,
{
  /// Construct a Bf from a slice of keys, returning Result.
  /// 从键切片构造 Bf，返回 Result。
  ///
  /// Accepts `Vec<T>`, `&Vec<T>`, `&[T]`, arrays, etc. via `AsRef`.
  /// 通过 `AsRef` 接受 `Vec<T>`、`&Vec<T>`、`&[T]`、数组等。
  pub fn try_from_keys(keys: impl AsRef<[T]>) -> crate::Result<Self> {
    Ok(Self {
      filter: Base::try_from_vec(hash_sort_dedup::<H, T>(keys.as_ref()))?,
      _phantom: PhantomData,
    })
  }

  /// Construct a Bf from an iterator of keys, returning Result.
  /// 从键迭代器构造 Bf，返回 Result。
  pub fn try_from_iterator<I>(iter: I) -> crate::Result<Self>
  where
    I: IntoIterator<Item = T>,
  {
    let mut hashes: Vec<u64> = iter.into_iter().map(|k| hash::<H, T>(&k)).collect();
    sort_dedup(&mut hashes);
    Ok(Self {
      filter: Base::try_from_vec(hashes)?,
      _phantom: PhantomData,
    })
  }

  /// Construct a Bf from a vector of keys, returning Result.
  /// 从键向量构造 Bf，返回 Result。
  pub fn try_from_vec(v: Vec<T>) -> crate::Result<Self> {
    Self::try_from_iterator(v)
  }
}

impl<T: ?Sized, U, H> Bf<T, Base<U>, H>
where
  U: Fingerprint,
{
  /// Construct a Bf from pre-computed u64 hashes, returning Result.
  /// 从预计算哈希构造 Bf，返回 Result。
  pub fn try_from_hashes(mut hashes: Vec<u64>) -> crate::Result<Self> {
    sort_dedup(&mut hashes);
    Ok(Self {
      filter: Base::try_from_vec(hashes)?,
      _phantom: PhantomData,
    })
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
  /// Accepts `Vec<T>`, `&Vec<T>`, `&[T]`, arrays, etc. via `AsRef`.
  /// 通过 `AsRef` 接受 `Vec<T>`、`&Vec<T>`、`&[T]`、数组等。
  pub fn from_slice(keys: impl AsRef<[T]>) -> Self
  where
    F: From<Vec<u64>>,
  {
    Self {
      filter: F::from(hash_sort_dedup::<H, T>(keys.as_ref())),
      _phantom: PhantomData,
    }
  }

  /// Construct a Bf from an iterator of keys.
  /// 从键迭代器构造 Bf。
  pub fn from_iterator<I>(iter: I) -> Self
  where
    F: From<Vec<u64>>,
    I: IntoIterator<Item = T>,
  {
    let mut hashes: Vec<u64> = iter.into_iter().map(|k| hash::<H, T>(&k)).collect();
    sort_dedup(&mut hashes);
    Self {
      filter: F::from(hashes),
      _phantom: PhantomData,
    }
  }

  /// Construct a Bf from a vector of keys.
  /// 从键向量构造 Bf。
  pub fn from_vec(v: Vec<T>) -> Self
  where
    F: From<Vec<u64>>,
  {
    Self::from_iterator(v)
  }
}

impl<T: ?Sized, F, H> Bf<T, F, H> {
  /// Construct a Bf directly from a vector of pre-computed u64 hashes.
  /// 直接从预先计算的 u64 哈希向量构造 Bf。
  ///
  /// Automatically sorts and deduplicates hashes.
  /// 自动对哈希进行排序和去重。
  pub fn from_hashes(mut hashes: Vec<u64>) -> Self
  where
    F: From<Vec<u64>>,
  {
    sort_dedup(&mut hashes);
    Self {
      filter: F::from(hashes),
      _phantom: PhantomData,
    }
  }

  /// Reinterprets the key type of the filter with zero runtime cost.
  /// 零成本重新解释过滤器的键类型。
  #[inline(always)]
  pub fn cast<NewT: ?Sized>(self) -> Bf<NewT, F, H> {
    Bf {
      filter: self.filter,
      _phantom: PhantomData,
    }
  }
}

impl<T, F, H> From<Vec<T>> for Bf<T, F, H>
where
  T: Hash,
  H: Hasher + Default,
  F: Filter<u64> + From<Vec<u64>>,
{
  fn from(v: Vec<T>) -> Self {
    Self::from_iterator(v)
  }
}

impl<T, F, H> From<&Vec<T>> for Bf<T, F, H>
where
  T: Hash,
  H: Hasher + Default,
  F: Filter<u64> + From<Vec<u64>>,
{
  fn from(v: &Vec<T>) -> Self {
    Self::from_slice(v.as_slice())
  }
}

impl<T, F, H> From<&[T]> for Bf<T, F, H>
where
  T: Hash,
  H: Hasher + Default,
  F: Filter<u64> + From<Vec<u64>>,
{
  fn from(keys: &[T]) -> Self {
    Self::from_slice(keys)
  }
}

impl<T, F, H> FromIterator<T> for Bf<T, F, H>
where
  T: Hash,
  H: Hasher + Default,
  F: Filter<u64> + From<Vec<u64>>,
{
  fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
    Self::from_iterator(iter)
  }
}

impl<T: ?Sized, F, H> Bf<T, F, H>
where
  H: Hasher + Default,
  F: Filter<u64>,
{
  /// Returns the total memory footprint in bytes.
  /// 返回该过滤器的总内存占用（字节数）。
  #[inline(always)]
  pub fn bytes(&self) -> usize {
    self.filter.bytes()
  }

  /// Returns the total memory footprint in bits.
  /// 返回该过滤器的总内存占用（位数）。
  #[inline(always)]
  pub fn bits(&self) -> usize {
    self.filter.bits()
  }

  /// Checks if a pre-computed hash exists in the underlying filter.
  /// 直接检查预先计算的哈希值是否存在于底层过滤器中。
  #[inline(always)]
  pub fn has_hash(&self, hash: u64) -> bool {
    self.filter.has(&hash)
  }

  /// Batch checks membership for multiple keys with 4-way instruction-level parallelism.
  /// 4路指令级并行批量检查多个键的成员资格。
  #[inline]
  pub fn contains_batch<Q>(&self, keys: &[&Q], results: &mut [bool])
  where
    T: Borrow<Q>,
    Q: Hash + ?Sized,
  {
    debug_assert_eq!(keys.len(), results.len());
    let (chunks, remainder_keys) = keys.as_chunks::<4>();
    let (res_chunks, remainder_results) = results.as_chunks_mut::<4>();

    for (k_chunk, r_chunk) in chunks.iter().zip(res_chunks.iter_mut()) {
      let h0 = hash::<H, Q>(k_chunk[0]);
      let h1 = hash::<H, Q>(k_chunk[1]);
      let h2 = hash::<H, Q>(k_chunk[2]);
      let h3 = hash::<H, Q>(k_chunk[3]);

      r_chunk[0] = self.filter.has(&h0);
      r_chunk[1] = self.filter.has(&h1);
      r_chunk[2] = self.filter.has(&h2);
      r_chunk[3] = self.filter.has(&h3);
    }

    for (&key, res) in remainder_keys.iter().zip(remainder_results) {
      *res = self.has(key);
    }
  }

  /// Batch checks membership for multiple keys, returning a Vec of booleans.
  /// 批量检查多个键的成员资格，返回布尔值 Vec。
  #[inline]
  pub fn contains_batch_vec<Q>(&self, keys: &[&Q]) -> Vec<bool>
  where
    T: Borrow<Q>,
    Q: Hash + ?Sized,
  {
    let mut results = vec![false; keys.len()];
    self.contains_batch(keys, &mut results);
    results
  }

  /// Returns `true` if the underlying filter contains the specified key.
  /// 如果底层过滤器包含指定的键，则返回 `true`。
  ///
  /// Allows querying with borrowed types (e.g., `&str` for `String` keys).
  /// 允许使用借用类型进行查询（例如 `String` 键使用 `&str`）。
  #[inline(always)]
  pub fn has<Q>(&self, key: &Q) -> bool
  where
    T: Borrow<Q>,
    Q: Hash + ?Sized,
  {
    self.filter.has(&hash::<H, Q>(key))
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
  #[inline(always)]
  fn from(src: Bf<&T, F, H>) -> Self {
    src.cast()
  }
}

impl<F, H> From<Bf<String, F, H>> for Bf<str, F, H> {
  #[inline(always)]
  fn from(src: Bf<String, F, H>) -> Self {
    src.cast()
  }
}

impl<F, H> From<Bf<Vec<u8>, F, H>> for Bf<[u8], F, H> {
  #[inline(always)]
  fn from(src: Bf<Vec<u8>, F, H>) -> Self {
    src.cast()
  }
}
