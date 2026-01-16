//! Implements BinaryFuse8 filters.

use crate::base::{BinaryFuse, BinaryFuseRef};

/// A `BinaryFuse8` filter.
///
/// ```
/// # extern crate alloc;
/// use jdb_xorf::{Filter, BinaryFuse8};
/// # use alloc::vec::Vec;
/// # use rand::Rng;
///
/// # let mut rng = rand::rng();
/// const SAMPLE_SIZE: usize = 1_000_000;
/// let keys: Vec<u64> = (0..SAMPLE_SIZE).map(|_| rng.random()).collect();
/// let filter = BinaryFuse8::from(&keys);
///
/// // no false negatives
/// for key in keys {
///     assert!(filter.contains(&key));
/// }
/// ```
pub type BinaryFuse8 = BinaryFuse<u8>;

/// A `BinaryFuse8Ref` filter.
pub type BinaryFuse8Ref<'a> = BinaryFuseRef<'a, u8>;
