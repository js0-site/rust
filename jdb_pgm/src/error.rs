use std::{io, result};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum PgmError {
  #[error("IO error: {0}")]
  Io(#[from] io::Error),
  #[error("Serialization error: {0}")]
  Serialization(String),
  #[error("Invalid data: {0}")]
  InvalidData(String),
}

pub type Result<T> = result::Result<T, PgmError>;
