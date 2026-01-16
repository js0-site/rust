use thiserror::Error;

/// DoQ 错误类型
#[derive(Error, Debug)]
pub enum Error {
  #[error(transparent)]
  Connection(#[from] quinn::ConnectionError),

  #[error(transparent)]
  Connect(#[from] quinn::ConnectError),

  #[error(transparent)]
  Write(#[from] quinn::WriteError),

  #[error(transparent)]
  Closed(#[from] quinn::ClosedStream),

  #[error(transparent)]
  Read(#[from] quinn::ReadToEndError),

  #[error(transparent)]
  Io(#[from] std::io::Error),

  #[error(transparent)]
  TlsConfig(#[from] quinn::crypto::rustls::NoInitialCipherSuite),

  #[error(transparent)]
  TlsVersion(#[from] rustls::Error),

  #[error(transparent)]
  Parse(#[from] dns_parse::Error),

  #[error("timeout")]
  Timeout,

  #[error("invalid address: {0}")]
  InvalidAddress(String),

  #[error("length mismatch")]
  LengthMismatch,
}

pub type Result<T> = std::result::Result<T, Error>;
