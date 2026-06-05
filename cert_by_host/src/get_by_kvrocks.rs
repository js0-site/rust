use std::{io::Cursor, result};

use fred::interfaces::KeysInterface;
use x509_parser::prelude::{FromDer, X509Certificate};
use xkv::R;

use crate::{DeadlineTs, Error, Result, SslConfig};

const SSL_KEY_PREFIX: &str = "ssl:";

async fn get_key_cert(host: &str) -> Result<Option<(String, String)>> {
  if let Some::<Vec<u8>>(data) = R.get(format!("{SSL_KEY_PREFIX}{host}")).await? {
    return Ok(sonic_rs::from_slice(&data)?);
  }
  Ok(None)
}

pub async fn get_by_kvrocks(host: &str) -> Result<Option<(SslConfig, DeadlineTs)>> {
  let Some((key_pem, cert_pem)) = get_key_cert(host).await? else {
    return Ok(None);
  };

  let key =
    rustls_pemfile::private_key(&mut Cursor::new(key_pem))?.ok_or(Error::InvalidPrivateKey)?;

  let cert: Vec<_> =
    rustls_pemfile::certs(&mut Cursor::new(cert_pem)).collect::<result::Result<_, _>>()?;

  let leaf = cert.first().ok_or(Error::CertChainEmpty)?;

  let (_, x509) = X509Certificate::from_der(leaf.as_ref()).map_err(|_| Error::X509Parse)?;

  // Use 0 for invalid timestamps / 无效时间戳使用 0
  let deadline = x509.validity().not_after.timestamp().max(0) as u64;

  Ok(Some((SslConfig { key, cert }, deadline)))
}
