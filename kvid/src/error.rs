use std::result;

use fred::error::Error as FredError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
  #[error("segment empty")]
  Empty,
  #[error("step overflow: {0}")]
  StepOverflow(u64),
  #[error(transparent)]
  Kv(#[from] FredError),
}

pub type Result<T> = result::Result<T, Error>;
