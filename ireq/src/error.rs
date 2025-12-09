use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
  #[error("HTTP Status Error")]
  Status(reqwest::Response),
  #[error(transparent)]
  Reqwest(#[from] reqwest::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
