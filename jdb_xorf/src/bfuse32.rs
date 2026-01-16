//! Implements BinaryFuse32 filters.

use crate::base::{BinaryFuse, BinaryFuseRef};

/// A `BinaryFuse32` filter.
///
/// ```
/// # extern crate alloc;
/// use jdb_xorf::{Filter, BinaryFuse32};
/// # use alloc::vec::Vec;
/// # use rand::Rng;
///
/// # let mut rng = rand::rng();
/// const SAMPLE_SIZE: usize = 1_000_000;
/// let keys: Vec<u64> = (0..SAMPLE_SIZE).map(|_| rng.random()).collect();
/// let filter = BinaryFuse32::from(&keys);
///
/// // no false negatives
/// for key in keys {
///     assert!(filter.contains(&key));
/// }
/// ```
pub type BinaryFuse32 = BinaryFuse<u32>;

/// A `BinaryFuse32Ref` filter.
pub type BinaryFuse32Ref<'a> = BinaryFuseRef<'a, u32>;
