use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
  #[error("DoH error from {doh}: {msg}")]
  Doh { doh: String, msg: String },

  #[error("HTTP request error: {0}")]
  Request(#[from] ireq::Error),

  #[error("JSON parsing error: {0}")]
  Json(String),

  #[error("DNS resolution failed: {0}")]
  Resolution(String),

  #[error("Invalid data format: {0}")]
  InvalidData(String),
}

pub type Result<T> = std::result::Result<T, Error>;
