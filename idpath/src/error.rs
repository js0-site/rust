use std::{
  error,
  fmt::{self, Display, Formatter},
  result,
};

use hipstr::HipStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
  InvalidPath(HipStr<'static>),
  DecodeFailed(HipStr<'static>),
}

impl error::Error for Error {}

impl Display for Error {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      Error::InvalidPath(path) => write!(f, "idpath : Invalid Path: {path}"),
      Error::DecodeFailed(path) => write!(f, "idpath : Decode Base32 Error: {path}"),
    }
  }
}

pub type Result<T> = result::Result<T, Error>;
