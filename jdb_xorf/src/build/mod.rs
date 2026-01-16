//! Implements a Builder abstraction for constructing filters from arbitrary types.

use alloc::vec::Vec;
use core::{
  borrow::Borrow,
  hash::{Hash, Hasher},
};

#[cfg(feature = "bitcode")]
use bitcode::{Decode, Encode};

use crate::{Filter, RapidHasher};

/// Builder for creating and querying filters with arbitrary key types.
///
/// A `Build` wraps an underlying `Filter<u64>` construction and provides automatic
/// hashing and deduplication for arbitrary keys (e.g., `String`, `&[u8]`).
///
/// ```
/// # extern crate alloc;
/// use jdb_xorf::{Filter, Build, BinaryFuse8};
/// # use alloc::vec::Vec;
/// # use rand::distr::Alphanumeric;
/// # use rand::Rng;
///
/// const SAMPLE_SIZE: usize = 1_000_000;
/// let passwords: Vec<String> = (0..SAMPLE_SIZE)
///     .map(|_| rand::rng().sample_iter(&Alphanumeric).take(30).map(char::from).collect())
///     .collect();
///
/// // Build enables safe construction from arbitrary types with auto-deduplication.
/// let pw_filter: Build<String, BinaryFuse8> = Build::from(&passwords);
///
/// for password in passwords {
///     assert!(pw_filter.contains(&password));
/// }
/// ```
#[cfg_attr(feature = "bitcode", derive(Decode, Encode))]
pub struct Build<T, F, H = RapidHasher>
where
  T: Hash,
  H: Hasher + Default,
  F: Filter<u64>,
{
  /// The underlying filter
  pub filter: F,
  /// Marker for the hasher type
  pub _hasher: core::marker::PhantomData<H>,
  /// Marker for the key type
  pub _type: core::marker::PhantomData<T>,
}

/// Computes the hash value for a key.
#[inline(always)]
fn hash<H: Hasher + Default, T: Hash + ?Sized>(key: &T) -> u64 {
  let mut hasher = H::default();
  key.hash(&mut hasher);
  hasher.finish()
}

impl<T, F, H> Filter<T> for Build<T, F, H>
where
  T: Hash,
  H: Hasher + Default,
  F: Filter<u64>,
{
  fn contains(&self, key: &T) -> bool {
    self.filter.contains(&hash::<H, T>(key))
  }

  fn len(&self) -> usize {
    self.filter.len()
  }
}

impl<T, F, H> Build<T, F, H>
where
  T: Hash,
  H: Hasher + Default,
  F: Filter<u64>,
{
  /// Returns `true` if the underlying filter contains the specified key.
  ///
  /// Allows querying with borrowed types (e.g., `&str` for `String` keys).
  pub fn contains<Q>(&self, key: &Q) -> bool
  where
    T: Borrow<Q>,
    Q: Hash + ?Sized,
  {
    let borrowed: &Q = key.borrow();
    self.filter.contains(&hash::<H, Q>(borrowed))
  }

  /// Construct a Build from a slice of keys.
  ///
  /// Automatically handles hashing, sorting, and deduplication to ensure
  /// filter construction succeeds.
  ///
  /// # Panics
  /// Only in the extremely unlikely event that the underlying filter fails to build even after deduplication.
  pub fn from(keys: &[T]) -> Self
  where
    F: TryFrom<Vec<u64>>,
  {
    let mut keys: Vec<u64> = keys.iter().map(hash::<H, T>).collect();
    keys.sort_unstable();
    keys.dedup();
    match F::try_from(keys) {
      Ok(filter) => Self {
        filter,
        _hasher: core::marker::PhantomData,
        _type: core::marker::PhantomData,
      },
      Err(_) => panic!("Failed to construct Build"),
    }
  }
}

impl<T, F, H> From<&Vec<T>> for Build<T, F, H>
where
  T: Hash,
  H: Hasher + Default,
  F: Filter<u64> + TryFrom<Vec<u64>>,
{
  fn from(v: &Vec<T>) -> Self {
    Self::from(v.as_slice())
  }
}

impl<T, F, H> From<&[T]> for Build<T, F, H>
where
  T: Hash,
  H: Hasher + Default,
  F: Filter<u64> + TryFrom<Vec<u64>>,
{
  fn from(keys: &[T]) -> Self {
    Self::from(keys)
  }
}
