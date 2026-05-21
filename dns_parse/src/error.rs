use std::result;

use thiserror::Error;

/// DNS 解析错误类型
#[derive(Error, Debug)]
pub enum Error {
  #[error("response too short")]
  ResponseTooShort,

  #[error("incomplete data")]
  IncompleteData,

  #[error("name out of bounds")]
  NameOutOfBounds,

  #[error("pointer out of bounds")]
  PointerOutOfBounds,
}

pub type Result<T> = result::Result<T, Error>;
