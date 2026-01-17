use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
  #[error("segment empty")]
  Empty,
  #[error("step overflow: {0}")]
  StepOverflow(u64),
  #[error(transparent)]
  Kv(#[from] fred::error::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
