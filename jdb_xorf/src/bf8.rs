//! Implements Bf8 filters.

use crate::base::{Bf, BfRef};

/// A `Bf8` filter.
///
/// ```
/// # extern crate alloc;
/// use jdb_xorf::{Filter, Bf8};
/// # use alloc::vec::Vec;
/// # use rand::Rng;
///
/// # let mut rng = rand::rng();
/// const SAMPLE_SIZE: usize = 1_000_000;
/// let keys: Vec<u64> = (0..SAMPLE_SIZE).map(|_| rng.random()).collect();
/// let filter = Bf8::from(&keys);
///
/// // no false negatives
/// for key in keys {
///     assert!(filter.contains(&key));
/// }
/// ```
pub type Bf8 = Bf<u8>;

/// A `Bf8Ref` filter.
pub type Bf8Ref<'a> = BfRef<'a, u8>;
