use std::result;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
  #[error(transparent)]
  Base64(#[from] base64::DecodeError),
  #[error(transparent)]
  Vb(#[from] vb::Error),
  #[error("TxtInvalid")]
  TxtInvalid,
}

pub type Result<T> = result::Result<T, Error>;
