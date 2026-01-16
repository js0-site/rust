//! Implements Bf32 filters.

use crate::base::{Bf, BfRef};

/// A `Bf32` filter.
///
/// ```
/// # extern crate alloc;
/// use jdb_xorf::{Filter, Bf32};
/// # use alloc::vec::Vec;
/// # use rand::Rng;
///
/// # let mut rng = rand::rng();
/// const SAMPLE_SIZE: usize = 1_000_000;
/// let keys: Vec<u64> = (0..SAMPLE_SIZE).map(|_| rng.random()).collect();
/// let filter = Bf32::from(&keys);
///
/// // no false negatives
/// for key in keys {
///     assert!(filter.contains(&key));
/// }
/// ```
pub type Bf32 = Bf<u32>;

/// A `Bf32Ref` filter.
pub type Bf32Ref<'a> = BfRef<'a, u32>;
