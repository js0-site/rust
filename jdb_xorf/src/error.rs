//! Error types for xor filters.
//! xor filter 的错误类型

use core::fmt;

/// Errors that can occur when constructing a filter.
/// 构造 filter 时可能发生的错误
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
  /// Failed to construct binary fuse filter.
  /// 构造 binary fuse filter 失败
  ConstructionFailed,
}

/// Result type for xor filter operations.
/// xor filter 操作的结果类型
pub type Result<T> = core::result::Result<T, Error>;

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
