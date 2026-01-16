//! Implements BinaryFuse16 filters.

use crate::base::{BinaryFuse, BinaryFuseRef};

/// A `BinaryFuse16` filter.
///
/// ```
/// # extern crate alloc;
/// use jdb_xorf::{Filter, BinaryFuse16};
/// # use alloc::vec::Vec;
/// # use rand::Rng;
///
/// # let mut rng = rand::rng();
/// const SAMPLE_SIZE: usize = 1_000_000;
/// let keys: Vec<u64> = (0..SAMPLE_SIZE).map(|_| rng.random()).collect();
/// let filter = BinaryFuse16::from(&keys);
///
/// // no false negatives
/// for key in keys {
///     assert!(filter.contains(&key));
/// }
/// ```
pub type BinaryFuse16 = BinaryFuse<u16>;

/// A `BinaryFuse16Ref` filter.
pub type BinaryFuse16Ref<'a> = BinaryFuseRef<'a, u16>;
