use thiserror::Error;

use crate::pb::DecodeError;

#[derive(Error, Debug)]
pub enum Error {
  #[error("too many calls in one request")]
  TooManyCalls,

  #[error(transparent)]
  DecodeError(#[from] DecodeError),
}
