use thiserror::Error;

/// Error enum for add_space crate.
/// add_space 库的错误枚举
#[derive(Error, Debug)]
pub enum Error {
  /// I/O error wrapper
  /// I/O 错误包装
  #[error(transparent)]
  Io(#[from] std::io::Error),
}

/// Result type alias for add_space crate.
/// add_space 库的 Result 类型别名
pub type Result<T, E = Error> = std::result::Result<T, E>;
