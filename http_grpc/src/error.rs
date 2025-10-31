use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
  #[error("too many calls in one request")]
  TooManyCalls,

  #[error(transparent)]
  DecodeError(#[from] crate::pb::DecodeError),
}
