use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DnsError {
  #[error("IO error: {0}")]
  Io(#[from] io::Error),
  #[error("message buffer too short")]
  BufferTooShort,
  #[error("name compression loop detected")]
  CompressionLoop,
  #[error("invalid label length")]
  InvalidLabelLength,
  #[error("invalid pointer target")]
  InvalidPointerTarget,
  #[error("invalid data")]
  InvalidData,
  #[error("DNS resolution failed")]
  ResolutionFailed,
  #[error("server response error")]
  ServerResponseError,
}

impl From<DnsError> for io::Error {
  fn from(e: DnsError) -> Self {
    match e {
      DnsError::Io(e) => e,
      _ => io::Error::new(io::ErrorKind::InvalidData, e),
    }
  }
}
