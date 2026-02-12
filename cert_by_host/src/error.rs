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
  #[error("cert chain empty")]
  CertChainEmpty,
  #[error("x509 parse failed")]
  X509Parse,
}

pub type Result<T> = std::result::Result<T, Error>;
