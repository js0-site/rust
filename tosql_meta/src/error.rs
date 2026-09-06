use std::{result, string};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
  #[error("empty binary data")]
  EmptyData,

  #[error("unexpected end of data while parsing field names")]
  TruncatedFieldNames,

  #[error("invalid UTF-8 in field name: {0}")]
  InvalidUtf8(#[from] string::FromUtf8Error),

  #[error("invalid Kind value: {0}")]
  InvalidKindValue(u8),
}

pub type Result<T> = result::Result<T, Error>;
