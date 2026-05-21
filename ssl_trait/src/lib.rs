#![cfg_attr(docsrs, feature(doc_cfg))]

use std::{
  borrow::Borrow,
  hash::{Hash, Hasher},
};

use anyhow::Result;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};

#[derive(Debug)]
pub struct SslConfig {
  pub key: PrivateKeyDer<'static>,
  pub cert: Vec<CertificateDer<'static>>,
}

impl PartialEq for SslConfig {
  fn eq(&self, other: &Self) -> bool {
    self.key.secret_der() == other.key.secret_der()
  }
}

impl Eq for SslConfig {}

impl Hash for SslConfig {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.key.secret_der().hash(state);
  }
}

pub trait CertByHost: Clone + Send + Sync + 'static {
  type Item: Borrow<SslConfig> + Send;
  fn get(&self, host: &str) -> impl Future<Output = Result<Option<Self::Item>>> + Send;
}
