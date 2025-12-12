use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
  #[error("DoH error from {doh}: {msg}")]
  Doh { doh: Box<str>, msg: Box<str> },

  #[error("HTTP request error: {0}")]
  Request(Box<ireq::Error>),

  #[error("JSON parsing error: {0}")]
  Json(Box<str>),

  #[error("DNS resolution failed: {0}")]
  Resolution(Box<str>),

  #[error("Invalid data format: {0}")]
  InvalidData(Box<str>),
}

impl From<ireq::Error> for Error {
  fn from(err: ireq::Error) -> Self {
    Error::Request(Box::new(err))
  }
}

pub type Result<T> = std::result::Result<T, Error>;
