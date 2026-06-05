use std::{io, result};

use thiserror::Error;

/// DoT 错误类型
#[derive(Error, Debug)]
pub enum Error {
  #[error(transparent)]
  Io(#[from] io::Error),

  #[error(transparent)]
  Parse(#[from] dns_parse::Error),

  #[error("timeout")]
  Timeout,

  #[error("invalid address: {0}")]
  InvalidAddress(String),

  #[error("invalid response length")]
  InvalidLength,

  #[error("response id mismatch")]
  IdMismatch,
}

pub type Result<T> = result::Result<T, Error>;
