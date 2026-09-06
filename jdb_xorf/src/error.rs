//! Error types for the crate.
//! 本 crate 的错误类型定义。

use core::result::Result as CoreResult;

use thiserror::Error;

/// Filter operation errors.
/// 过滤器操作错误。
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
  /// Construction failed after exceeding maximum iterations.
  /// 超过最大尝试次数后构建失败。
  #[error("filter construction failed after reaching maximum iterations")]
  BuildFailed,

  /// Duplicate keys are not allowed in Binary Fuse filter construction.
  /// Binary Fuse 过滤器构建不允许包含重复键。
  #[error("duplicate keys are not allowed")]
  DuplicateKeys,
}

/// Specialized Result type for filter operations.
/// 过滤器操作专用的 Result 类型。
pub type Result<T> = CoreResult<T, Error>;
