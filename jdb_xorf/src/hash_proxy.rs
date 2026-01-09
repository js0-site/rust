//! Implements a hashing proxy for xor filters.

use alloc::vec::Vec;
use core::{
  borrow::Borrow,
  hash::{Hash, Hasher},
};

#[cfg(feature = "bitcode")]
use bitcode::{Decode, Encode};

use crate::Filter;

/// Arbitrary key type proxy for xor filters.
///
/// A `HashProxy` exposes a [`Filter`] trait for arbitrary key types, using a `Filter<u64>` as
/// an underlying keystore. The performance and collision rate of the `HashProxy` filter depends
/// on the choice of [`Hasher`] and underlying [`Filter`]. A `HashProxy` is immutable once
/// constructed.
///
/// ```
/// # extern crate alloc;
/// # extern crate std;
/// use std::collections::hash_map::DefaultHasher;
/// use jdb_xorf::{Filter, HashProxy, Xor8};
/// # use alloc::vec::Vec;
/// # use rand::distr::Alphanumeric;
/// # use rand::Rng;
///
/// const SAMPLE_SIZE: usize = 1_000_000;
/// let passwords: Vec<String> = (0..SAMPLE_SIZE)
///     .map(|_| rand::rng().sample_iter(&Alphanumeric).take(30).map(char::from).collect())
///     .collect();
///
/// let pw_filter: HashProxy<String, DefaultHasher, Xor8> = HashProxy::from(&passwords);
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
/// # extern crate std;
/// # use std::collections::hash_map::DefaultHasher;
/// # use jdb_xorf::{Filter, HashProxy, Xor8};
/// # use alloc::vec::Vec;
/// # use rand::Rng;
/// # use rand::distr::Alphanumeric;
/// #
/// # const SAMPLE_SIZE: usize = 1_000_000;
/// # let passwords: Vec<String> = (0..SAMPLE_SIZE)
/// #     .map(|_| rand::rng().sample_iter(&Alphanumeric).take(30).map(char::from).collect())
/// #     .collect();
/// #
/// let pw_filter: HashProxy<_, DefaultHasher, Xor8> = HashProxy::from(&passwords);
/// ```
///
/// `HashProxy` supports flexible queries using the `contains` method, similar to `HashMap`'s
/// `get` method. This allows querying with borrowed types. For example, a `HashProxy<String, ...>`
/// can be queried with `&str`:
///
/// ```
/// # extern crate alloc;
/// # extern crate std;
/// use std::collections::hash_map::DefaultHasher;
/// use jdb_xorf::{Filter, HashProxy, Xor8};
/// # use alloc::vec::Vec;
///
/// let fruits = vec!["apple".to_string(), "banana".to_string(), "orange".to_string()];
/// let filter: HashProxy<String, DefaultHasher, Xor8> = HashProxy::from(&fruits);
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
pub struct HashProxy<T, H, F>
where
  T: Hash,
  H: Hasher + Default,
  F: Filter<u64>,
{
  filter: F,
  _hasher: core::marker::PhantomData<H>,
  _type: core::marker::PhantomData<T>,
}

#[inline]
fn hash<H: Hasher + Default, T: Hash + ?Sized>(key: &T) -> u64 {
  let mut hasher = H::default();
  key.hash(&mut hasher);
  hasher.finish()
}

impl<T, H, F> Filter<T> for HashProxy<T, H, F>
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

impl<T, H, F> HashProxy<T, H, F>
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
  /// # extern crate std;
  /// use std::collections::hash_map::DefaultHasher;
  /// use jdb_xorf::{Filter, HashProxy, Xor8};
  /// # use alloc::vec::Vec;
  ///
  /// let fruits = vec!["apple".to_string(), "banana".to_string(), "orange".to_string()];
  /// let filter: HashProxy<String, DefaultHasher, Xor8> = HashProxy::from(&fruits);
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
}

impl<T, H, F> From<&[T]> for HashProxy<T, H, F>
where
  T: Hash,
  H: Hasher + Default,
  F: Filter<u64> + From<Vec<u64>>,
{
  fn from(keys: &[T]) -> Self {
    let keys: Vec<u64> = keys.iter().map(hash::<H, T>).collect();
    Self {
      filter: F::from(keys),
      _hasher: core::marker::PhantomData,
      _type: core::marker::PhantomData,
    }
  }
}

impl<T, H, F> From<&Vec<T>> for HashProxy<T, H, F>
where
  T: Hash,
  H: Hasher + Default,
  F: Filter<u64> + From<Vec<u64>>,
{
  fn from(v: &Vec<T>) -> Self {
    Self::from(v.as_slice())
  }
}

// TODO(ayazhafiz): We should support a `TryFrom` trait as well. Today this is impossible due to
// rustc's core blanket implementation of `Into`, which picks up a conflicting implementation when
// both `From<T>` and `TryFrom<T>` with unbound type parameters `T` are defined.
//
// See https://github.com/rust-lang/rust/issues/50133 for more details.

#[cfg(test)]
mod test {
  use alloc::{string::ToString, vec::Vec};

  use rand::{Rng, distr::Alphanumeric};

  use crate::{HashProxy, xor8::Xor8, xor16::Xor16, xor32::Xor32};

  extern crate std;
  use std::{collections::hash_map::DefaultHasher, string::String};

  #[test]
  fn test_initialization_from() {
    const SAMPLE_SIZE: usize = 1_000_000;
    // Key generation is expensive. Do it once and make copies during tests.
    let keys: Vec<String> = (0..SAMPLE_SIZE)
      .map(|_| {
        rand::rng()
          .sample_iter(&Alphanumeric)
          .take(15)
          .map(char::from)
          .collect()
      })
      .collect();

    macro_rules! drive_test {
      ($xorf:ident) => {{
        let keys = keys.clone();
        let filter: HashProxy<_, DefaultHasher, $xorf> = HashProxy::from(&keys);
        for key in keys {
          assert!(filter.contains(&key));
        }
      }};
    }

    drive_test!(Xor8);
    drive_test!(Xor16);
    drive_test!(Xor32);
  }

  #[test]
  fn test_borrow_query() {
    let keys = vec![
      "apple".to_string(),
      "banana".to_string(),
      "orange".to_string(),
    ];
    let filter: HashProxy<_, DefaultHasher, Xor8> = HashProxy::from(&keys);

    // Can query with &str instead of &String
    assert!(filter.contains("apple"));
    assert!(filter.contains("banana"));
    assert!(filter.contains("orange"));
    assert!(!filter.contains("pear"));
    assert!(!filter.contains("grape"));
  }
}
