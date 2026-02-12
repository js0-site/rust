//! # jdb_xorf
//!
//! Fast and compact Xor and Binary Fuse filters for Rust.
//!
//! 快速、紧凑的 Rust Xor 和 Binary Fuse 过滤器。
//!
//! Please refer to the [README](https://self) for detailed documentation.
//!
//! 详细文档请参阅 [README](https://self)。

#![allow(unexpected_cfgs)]
#![no_std]
#![cfg_attr(feature = "nightly", feature(allocator_internals), needs_allocator)]
#![warn(missing_docs)]
#![allow(clippy::multiple_crate_versions, clippy::fallible_impl_from)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[macro_use]
extern crate alloc;

#[macro_use]
mod prelude;
mod base;
mod hash;

pub use base::{Bf8, Bf16, Bf32};
mod bf;

use core::{borrow::Borrow, hash::Hash};

pub use bf::Bf;
pub use hash::{RapidHasher, mix64};

/// Methods common to xor filters.
///
/// Xor 过滤器的通用方法。
pub trait Filter<T: ?Sized> {
  /// Returns `true` if the filter probably contains the specified key.
  ///
  /// 如果过滤器可能包含指定的键，则返回 `true`。
  ///
  /// There can never be a false negative, but there is a small possibility of false positives.
  /// Refer to individual filters' documentation for false positive rates.
  ///
  /// 绝不会出现假阴性（False Negative），但有极小的假阳性（False Positive）可能性。
  /// 关于假阳性率，请参阅各个过滤器的文档。
  fn has<Q: ?Sized>(&self, key: &Q) -> bool
  where
    T: Borrow<Q>,
    Q: Hash;

  /// Returns the number of fingerprints in the filter.
  ///
  /// 返回过滤器中指纹的数量。
  fn len(&self) -> usize;

  /// Returns `true` if the filter has no fingerprints.
  ///
  /// 如果过滤器没有指纹，则返回 `true`。
  fn is_empty(&self) -> bool {
    self.len() == 0
  }
}
