use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
  #[error("cache error: {0}")]
  Cache(String),
}

pub type Result<T> = std::result::Result<T, Error>;
