//! Implements a hashing proxy for xor filters.

use alloc::vec::Vec;
use core::{
  borrow::Borrow,
  hash::{Hash, Hasher},
};

#[cfg(feature = "bitcode")]
use bitcode::{Decode, Encode};

use crate::{Filter, RapidHasher};

/// Arbitrary key type proxy for xor filters.
///
/// A `HashProxy` exposes a [`Filter`] trait for arbitrary key types, using a `Filter<u64>` as
/// an underlying keystore. The performance and collision rate of the `HashProxy` filter depends
/// on the choice of [`Hasher`] and underlying [`Filter`]. A `HashProxy` is immutable once
/// constructed.
///
/// ```
/// # extern crate alloc;
/// use jdb_xorf::{Filter, HashProxy, BinaryFuse8};
/// # use alloc::vec::Vec;
/// # use rand::distr::Alphanumeric;
/// # use rand::Rng;
///
/// const SAMPLE_SIZE: usize = 1_000_000;
/// let passwords: Vec<String> = (0..SAMPLE_SIZE)
///     .map(|_| rand::rng().sample_iter(&Alphanumeric).take(30).map(char::from).collect())
///     .collect();
///
/// // RapidHasher is used by default for ultra-high performance.
/// let pw_filter: HashProxy<String, BinaryFuse8> = HashProxy::try_from(&passwords).unwrap();
///
/// for password in passwords {
///     assert!(pw_filter.contains(&password));
/// }
/// ```
///
/// While a `HashProxy` persists type information about the keys it is constructed with, in most
/// cases the key type parameter can be elided. For example, the `pw_filter` defined above can also
/// be defined as
///
/// ```
/// # extern crate alloc;
/// # use jdb_xorf::{Filter, HashProxy, BinaryFuse8};
/// # use alloc::vec::Vec;
/// # use rand::Rng;
/// # use rand::distr::Alphanumeric;
/// #
/// # const SAMPLE_SIZE: usize = 1_000_000;
/// # let passwords: Vec<String> = (0..SAMPLE_SIZE)
/// #     .map(|_| rand::rng().sample_iter(&Alphanumeric).take(30).map(char::from).collect())
/// #     .collect();
/// #
/// let pw_filter: HashProxy<_, BinaryFuse8> = HashProxy::try_from(&passwords).unwrap();
/// ```
///
/// `HashProxy` supports flexible queries using the `contains` method, similar to `HashMap`'s
/// `get` method. This allows querying with borrowed types. For example, a `HashProxy<String, ...>`
/// can be queried with `&str`:
///
/// ```
/// # extern crate alloc;
/// use jdb_xorf::{Filter, HashProxy, BinaryFuse8};
/// # use alloc::vec::Vec;
///
/// let fruits = vec!["apple".to_string(), "banana".to_string(), "orange".to_string()];
/// let filter: HashProxy<String, BinaryFuse8> = HashProxy::try_from(&fruits).unwrap();
///
/// // Can query with &str instead of &String
/// assert!(filter.contains("apple"));
/// assert!(filter.contains("banana"));
/// assert!(!filter.contains("pear"));
/// ```
///
/// Serializing and deserializing `HashProxy`s can be enabled with the [`bitcode`] feature.
///
/// [`Filter`]: crate::Filter
/// [`Hasher`]: core::hash::Hasher
/// [`bitcode`]: https://github.com/SoftbearStudios/bitcode
#[cfg_attr(feature = "bitcode", derive(Decode, Encode))]
pub struct HashProxy<T, F, H = RapidHasher>
where
  T: Hash,
  H: Hasher + Default,
  F: Filter<u64>,
{
  filter: F,
  _hasher: core::marker::PhantomData<H>,
  _type: core::marker::PhantomData<T>,
}

#[inline(always)]
fn hash<H: Hasher + Default, T: Hash + ?Sized>(key: &T) -> u64 {
  let mut hasher = H::default();
  key.hash(&mut hasher);
  hasher.finish()
}

impl<T, F, H> Filter<T> for HashProxy<T, F, H>
where
  T: Hash,
  H: Hasher + Default,
  F: Filter<u64>,
{
  /// Returns `true` if the underlying filter contains the specified key.
  fn contains(&self, key: &T) -> bool {
    self.filter.contains(&hash::<H, T>(key))
  }

  fn len(&self) -> usize {
    self.filter.len()
  }
}

impl<T, F, H> HashProxy<T, F, H>
where
  T: Hash,
  H: Hasher + Default,
  F: Filter<u64>,
{
  /// Returns `true` if the underlying filter contains the specified key.
  ///
  /// This method accepts any type `Q` that can be borrowed from `T`, allowing for more flexible
  /// queries. For example, you can query a `HashProxy<String, ...>` with `&str`.
  ///
  /// # Examples
  ///
  /// ```
  /// # extern crate alloc;
  /// use jdb_xorf::{Filter, HashProxy, BinaryFuse8};
  /// # use alloc::vec::Vec;
  ///
  /// let fruits = vec!["apple".to_string(), "banana".to_string(), "orange".to_string()];
  /// let filter: HashProxy<String, BinaryFuse8> = HashProxy::try_from(&fruits).unwrap();
  ///
  /// // Can query with &str instead of &String
  /// assert!(filter.contains("apple"));
  /// assert!(filter.contains("banana"));
  /// assert!(!filter.contains("pear"));
  /// ```
  pub fn contains<Q>(&self, key: &Q) -> bool
  where
    T: Borrow<Q>,
    Q: Hash + ?Sized,
  {
    let borrowed: &Q = key.borrow();
    self.filter.contains(&hash::<H, Q>(borrowed))
  }

  /// Try to construct the filter from a slice of keys.
  ///
  /// This implementation automatically de-duplicates the keys to ensure high
  /// construction success rate.
  pub fn try_from(keys: &[T]) -> core::result::Result<Self, F::Error>
  where
    F: TryFrom<Vec<u64>>,
  {
    let mut keys: Vec<u64> = keys.iter().map(hash::<H, T>).collect();
    keys.sort_unstable();
    keys.dedup();
    F::try_from(keys).map(|filter| Self {
      filter,
      _hasher: core::marker::PhantomData,
      _type: core::marker::PhantomData,
    })
  }

  /// Try to construct the filter from a vector of keys.
  pub fn try_from_vec(v: &Vec<T>) -> core::result::Result<Self, F::Error>
  where
    F: TryFrom<Vec<u64>>,
  {
    Self::try_from(v.as_slice())
  }

  /// Construct the filter from a slice of keys, panicking on failure.
  ///
  /// This implementation automatically de-duplicates keys.
  ///
  /// # Panics
  ///
  /// Panics if the underlying filter fails to construct (extremely rare after de-duplication).
  pub fn new(keys: &[T]) -> Self
  where
    F: TryFrom<Vec<u64>>,
    F::Error: core::fmt::Debug,
  {
    Self::try_from(keys).expect("Failed to construct HashProxy")
  }
}

impl<T, F, H> TryFrom<&Vec<T>> for HashProxy<T, F, H>
where
  T: Hash,
  H: Hasher + Default,
  F: Filter<u64> + TryFrom<Vec<u64>>,
{
  type Error = F::Error;

  fn try_from(v: &Vec<T>) -> core::result::Result<Self, Self::Error> {
    Self::try_from(v.as_slice())
  }
}

impl<T, F, H> TryFrom<&[T]> for HashProxy<T, F, H>
where
  T: Hash,
  H: Hasher + Default,
  F: Filter<u64> + TryFrom<Vec<u64>>,
{
  type Error = F::Error;

  fn try_from(keys: &[T]) -> core::result::Result<Self, Self::Error> {
    Self::try_from(keys)
  }
}

// HashProxy primarily uses TryFrom for construction.

#[cfg(test)]
mod test {
  use alloc::{
    string::{String, ToString},
    vec::Vec,
  };

  use rand::Rng;

  use crate::{BinaryFuse8, BinaryFuse16, BinaryFuse32, Filter};

  extern crate std;
  use core::hash::{Hash, Hasher};
  use std::collections::hash_map::DefaultHasher;

  #[test]
  fn test_initialization_from() {
    const SAMPLE_SIZE: usize = 1_000_000;
    // Key generation is expensive. Do it once and make copies during tests.
    let keys: Vec<u64> = (0..SAMPLE_SIZE).map(|_| rand::rng().random()).collect();

    macro_rules! drive_test {
      ($xorf:ident) => {{
        let keys = keys.clone();
        let filter = $xorf::try_from(&keys).unwrap();
        for key in keys {
          assert!(filter.contains(&key));
        }
      }};
    }

    drive_test!(BinaryFuse8);
    drive_test!(BinaryFuse16);
    drive_test!(BinaryFuse32);
  }

  #[test]
  fn test_borrow_query() {
    let keys: Vec<String> = vec![
      "apple".to_string(),
      "banana".to_string(),
      "orange".to_string(),
    ];

    // Hash the keys to u64
    let hashed_keys: Vec<u64> = keys
      .iter()
      .map(|k: &String| {
        let mut hasher = DefaultHasher::default();
        k.hash(&mut hasher);
        hasher.finish()
      })
      .collect();

    // Create filter with hashed keys
    let filter = BinaryFuse8::try_from(&hashed_keys).unwrap();

    // Test with original keys
    assert!(filter.contains(&hashed_keys[0]));
    assert!(filter.contains(&hashed_keys[1]));
    assert!(filter.contains(&hashed_keys[2]));
  }

  #[test]
  fn test_duplicate_keys() {
    let keys = vec!["apple", "banana", "apple", "cherry", "banana"];
    // This should NOT panic or return Err because HashProxy de-duplicates
    let filter = crate::HashProxy::<&str, BinaryFuse8>::new(&keys);

    assert!(filter.contains("apple"));
    assert!(filter.contains("banana"));
    assert!(filter.contains("cherry"));
  }
}
