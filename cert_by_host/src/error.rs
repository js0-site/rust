use std::{io, result};

use fred::error::Error as FredError;

#[derive(thiserror::Error, Debug)]
pub enum Error {
  #[error(transparent)]
  Fred(#[from] FredError),
  #[error(transparent)]
  Sonic(#[from] sonic_rs::Error),
  #[error(transparent)]
  Io(#[from] io::Error),
  #[error("invalid private key")]
  InvalidPrivateKey,
  #[error("cert chain empty")]
  CertChainEmpty,
  #[error("x509 parse failed")]
  X509Parse,
}

pub type Result<T> = result::Result<T, Error>;
