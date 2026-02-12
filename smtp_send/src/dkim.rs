use std::sync::Arc;

use mail_send::mail_auth::{
  common::crypto::{RsaKey, Sha256},
  dkim::{DkimSigner, Done},
};
use papaya::HashMap;
use rsa::pkcs1::EncodeRsaPrivateKey;
use sk_dkim::Sk;

type DkimSignerType = DkimSigner<RsaKey<Sha256>, Done>;

/// RFC 6376: 每个头部列出 N+1 次防止注入攻击
/// RFC 6376: List each header N+1 times to prevent injection attacks
const DKIM_HEADERS: [&str; 11] = [
  "From",
  "From",
  "Subject",
  "Subject",
  "Date",
  "Date",
  "To",
  "To",
  "Cc",
  "Cc",
  "Message-ID",
];

/// 全局 DKIM signer 缓存
/// Global DKIM signer cache
#[static_init::dynamic(lazy)]
static CACHE: HashMap<String, Arc<DkimSignerType>> = HashMap::new();

#[allow(deprecated)]
pub fn signer(selector: &str, host: &str, sk: &Sk) -> Option<Arc<DkimSignerType>> {
  let key = format!("{selector}.{host}");
  let guard = CACHE.pin();

  if let Some(v) = guard.get(&key) {
    return Some(v.clone());
  }

  let dkim = sk.dkim(selector, host);
  let der = dkim.to_pkcs1_der().ok()?;
  let rsa_key = RsaKey::<Sha256>::from_der(der.as_bytes()).ok()?;

  let signer = Arc::new(
    DkimSigner::from_key(rsa_key)
      .domain(host)
      .selector(selector)
      .headers(DKIM_HEADERS),
  );

  guard.insert(key, signer.clone());
  Some(signer)
}
