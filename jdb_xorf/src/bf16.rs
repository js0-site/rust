//! Implements Bf16 filters.

use crate::base::{Bf, BfRef};

/// A `Bf16` filter.
///
/// ```
/// # extern crate alloc;
/// use jdb_xorf::{Filter, Bf16};
/// # use alloc::vec::Vec;
/// # use rand::Rng;
///
/// # let mut rng = rand::rng();
/// const SAMPLE_SIZE: usize = 1_000_000;
/// let keys: Vec<u64> = (0..SAMPLE_SIZE).map(|_| rng.random()).collect();
/// let filter = Bf16::from(&keys);
///
/// // no false negatives
/// for key in keys {
///     assert!(filter.contains(&key));
/// }
/// ```
pub type Bf16 = Bf<u16>;

/// A `Bf16Ref` filter.
pub type Bf16Ref<'a> = BfRef<'a, u16>;
