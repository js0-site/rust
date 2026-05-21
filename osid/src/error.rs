//! Error types / 错误类型

use std::io;

use thiserror::Error;

/// Machine ID error / 机器ID错误
#[derive(Debug, Error)]
pub enum Error {
  #[error("failed to create dir / 创建目录失败: {0}")]
  CreateDir(#[source] io::Error),
  #[error("failed to write id / 写入ID失败: {0}")]
  WriteId(#[source] io::Error),
}
