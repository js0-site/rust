use std::result;

use thiserror::Error;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
  #[error("HTTP error: {0}")]
  Http(#[from] ireq::Error),

  #[error("JSON parse error: {0}")]
  Json(#[from] serde_json::Error),
}
