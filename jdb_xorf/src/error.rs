//! Error types for xor filters.

use core::fmt;

/// Errors that can occur when constructing a filter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
  /// Failed to construct binary fuse filter.
  /// 构造 binary fuse filter 失败
  ConstructionFailed,
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::ConstructionFailed => {
        write!(f, "Failed to construct binary fuse filter.")
      }
    }
  }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}
