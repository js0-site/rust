//! A variant of [Cuckoo Filter][cuckoo filter] whose size automatically scales as necessary.
//! 一种可自动扩展大小的布谷鸟过滤器变体
//!
//! # Examples
//!
//! Basic usage:
//!
//! ```
//! use autoscale_cuckoo_filter::CuckooFilter;
//!
//! let mut filter = CuckooFilter::<str>::new(100, 0.001);
//! assert!(!filter.contains("foo"));
//! filter.add_if_not_exist("foo");
//! assert!(filter.contains("foo"));
//! ```
//!
//! Filter grows automatically:
//!
//! ```
//! use autoscale_cuckoo_filter::CuckooFilter;
//!
//! let mut filter = CuckooFilter::<usize>::new(100, 0.001);
//! assert_eq!(filter.capacity(), 128);
//!
//! for i in 0..1000 {
//!     filter.add_if_not_exist(&i);
//! }
//! assert_eq!(filter.capacity(), 1923);
//! ```
//!
//! # References
//!
//! - [Cuckoo Filter: Practically Better Than Bloom][cuckoo filter]
//! - [Scalable Bloom Filters][scalable bloom filters]
//!
//! [cuckoo filter]: https://www.cs.cmu.edu/~dga/papers/cuckoo-conext2014.pdf
//! [scalable bloom filters]: http://haslab.uminho.pt/cbm/files/dbloom.pdf
#![warn(missing_docs)]

pub use cuckoo_filter::{CuckooFilter, CuckooFilterBuilder, DefaultHasher};

mod base;
mod bits;
mod buckets;
mod cuckoo_filter;

use std::hash::{Hash, Hasher};

/// Compute hash for item.
/// 计算元素的哈希值
#[inline(always)]
pub(crate) fn hash<T: ?Sized + Hash, H: Hasher + Clone>(hasher: &H, item: &T) -> u64 {
  let mut h = hasher.clone();
  item.hash(&mut h);
  h.finish()
}
