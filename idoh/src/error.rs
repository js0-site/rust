

use thiserror::Error;
#[derive(Error, Debug)]
pub enum Error {
  #[error("dns query no result")]
  DnsNoResult,
  #[error(transparent)]
  Aok(#[from] aok::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
