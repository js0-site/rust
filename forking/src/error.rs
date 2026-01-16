use std::io;

/// Fork operation errors
/// 分叉操作错误
#[derive(thiserror::Error, Debug)]
pub enum Error {
  #[error("Fork failed")]
  ForkFailed,

  #[error("IO error: {0}")]
  Io(#[from] io::Error),
}
