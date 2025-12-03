#[derive(thiserror::Error, Debug)]
pub enum Error {
  #[error(transparent)]
  Fred(#[from] fred::error::Error),
  #[error(transparent)]
  Sonic(#[from] sonic_rs::Error),
  #[error(transparent)]
  Io(#[from] std::io::Error),
  #[error("invalid private key")]
  InvalidPrivateKey,
  #[error("invalid certificate")]
  InvalidCertificate,
  #[error("certificate chain is empty")]
  CertificateChainEmpty,
  #[error("failed to parse x509 certificate")]
  X509ParseError,
}

pub type Result<T> = std::result::Result<T, Error>;
