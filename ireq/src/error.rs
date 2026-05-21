use std::result;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
  #[error("HTTP Status Error")]
  Status(reqwest::Response),
  #[error(transparent)]
  Reqwest(#[from] reqwest::Error),
}

pub type Result<T> = result::Result<T, Error>;
