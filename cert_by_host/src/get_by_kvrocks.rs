use std::io::Cursor;

use fred::interfaces::KeysInterface;
use x509_parser::prelude::{FromDer, X509Certificate};
use xkv::R;

use crate::{DeadlineTs, Error, Result, SslConfig};

pub type Key = String;
pub type Cert = String;

pub async fn get_key_cert(host: impl AsRef<str>) -> Result<Option<(Key, Cert)>> {
  let host = host.as_ref();
  if let Some::<Vec<u8>>(key_cert) = R.get(format!("ssl:{host}")).await? {
    return Ok(sonic_rs::from_slice(&key_cert)?);
  }
  Ok(None)
}

pub async fn get_by_kvrocks(host: impl Into<String>) -> Result<Option<(SslConfig, DeadlineTs)>> {
  let host = host.into();
  if let Some((key, cert)) = get_key_cert(&host).await? {
    let mut key = Cursor::new(key);

    let key = rustls_pemfile::private_key(&mut key)?.ok_or(Error::InvalidPrivateKey)?;

    let mut cert_reader = Cursor::new(cert);

    let cert =
      rustls_pemfile::certs(&mut cert_reader).collect::<std::result::Result<Vec<_>, _>>()?;

    let leaf = cert
      .first()
      .ok_or(crate::error::Error::CertificateChainEmpty)?;

    let (_, x509) =
      X509Certificate::from_der(leaf.as_ref()).map_err(|_| crate::error::Error::X509ParseError)?;

    let deadline = x509.validity().not_after.timestamp();

    return Ok(Some((SslConfig { key, cert }, deadline as u64)));
  }

  Ok(None)
}
