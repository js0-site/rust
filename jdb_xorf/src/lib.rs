//! # jdb_xorf
//!
//! Fast and compact Xor and Binary Fuse filters for Rust.
//!
//! 快速、紧凑的 Rust Xor 和 Binary Fuse 过滤器。
//!
//! Please refer to the [README](https://self) for detailed documentation.
//!
//! 详细文档请参阅 [README](https://self)。

#![cfg_attr(docsrs, feature(doc_cfg))]

extern crate alloc;

mod prelude;
mod base;
mod bf;
pub mod error;
mod hash;

use core::{borrow::Borrow, hash::Hash};

pub use base::{Base, Bf8, Bf16, Bf32, Bf64};
pub use bf::Bf;
pub use error::{Result, Error};
pub use prelude::bfuse::Desc;
#[cfg(feature = "gxhash")]
pub use hash::GxHasher;
pub use hash::MixHasher;
#[cfg(feature = "museair")]
pub use hash::MuseairHasher;
pub use hash::{DefaultHasher, Fingerprint, hash_key, mix_key, mix64};

#[cfg(debug_assertions)]
pub(crate) use prelude::all_distinct;
pub(crate) use prelude::{
  batch_chunks_4, batch_vec,
  bfuse::{hash_of_hash, mod3, seg_len, size_factor},
  make_fp_block,
};

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
  fn has<Q>(&self, key: &Q) -> bool
  where
    Q: ?Sized + Hash,
    T: Borrow<Q>;

  /// Directly queries a u64 key without reference or size checks.
  /// 直接查询 u64 键的高速路径（避免引用开销）。
  #[inline(always)]
  fn has_key(&self, key: u64) -> bool
  where
    T: Borrow<u64>,
  {
    self.has(&key)
  }

  /// Batch checks membership for multiple u64 keys.
  /// 批量检查多个 u64 键的成员资格。
  #[inline]
  fn contains_batch_keys(&self, keys: &[u64], results: &mut [bool])
  where
    T: Borrow<u64>,
  {
    debug_assert_eq!(keys.len(), results.len());
    for (k, r) in keys.iter().zip(results.iter_mut()) {
      *r = self.has_key(*k);
    }
  }

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

  /// Returns approximate memory usage in bytes.
  ///
  /// 返回占用的内存字节数。
  fn bytes(&self) -> usize;

  /// Returns approximate memory usage in bits.
  ///
  /// 返回占用的内存位数。
  fn bits(&self) -> usize {
    self.bytes().saturating_mul(8)
  }
}
