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
const DKIM_HEADERS: [&str; 15] = [
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
  "List-Unsubscribe",
  "List-Unsubscribe",
  "List-Unsubscribe-Post",
  "List-Unsubscribe-Post",
];

/// 全局 DKIM signer 缓存
/// Global DKIM signer cache
#[static_init::dynamic(lazy)]
static CACHE: HashMap<String, Arc<DkimSignerType>> = HashMap::new();

pub fn signer(selector: &str, host: &str, sk: &Sk) -> Option<Arc<DkimSignerType>> {
  let key = format!("{selector}.{host}");
  let guard = CACHE.pin();

  if let Some(v) = guard.get(&key) {
    return Some(v.clone());
  }

  let dkim = sk.dkim(selector, host);
  let der = dkim.to_pkcs1_der().ok()?;
  let rsa_key = RsaKey::<Sha256>::from_key_der(rustls_pki_types::PrivateKeyDer::Pkcs1(
    rustls_pki_types::PrivatePkcs1KeyDer::from(der.as_bytes()),
  ))
  .ok()?;

  let signer = Arc::new(
    DkimSigner::from_key(rsa_key)
      .domain(host)
      .selector(selector)
      .headers(DKIM_HEADERS),
  );

  guard.insert(key, signer.clone());
  Some(signer)
}

#[cfg(test)]
mod tests {
  use mail_send::mail_builder::MessageBuilder;

  use super::*;

  #[test]
  fn test_dkim_headers_in_signature() {
    let sk = Sk::new(b"dummy_secret_key_12345678901234567890123456789012");
    let signer = signer("selector", "example.com", &sk).unwrap();

    let message = MessageBuilder::new()
      .from(("Test", "test@example.com"))
      .to("to@example.com")
      .subject("Subject")
      .header(
        "List-Unsubscribe",
        mail_send::mail_builder::headers::text::Text::new("<https://example.com/unsubscribe>"),
      )
      .header(
        "List-Unsubscribe-Post",
        mail_send::mail_builder::headers::text::Text::new("List-Unsubscribe=One-Click"),
      )
      .text_body("Body");

    let mut output = Vec::new();
    message.write_to(&mut output).unwrap();

    let signature = signer.sign(&output).unwrap();
    let signature_str = signature.to_string().to_lowercase();

    assert!(
      signature_str.contains("list-unsubscribe"),
      "Signature should contain list-unsubscribe header"
    );
    assert!(
      signature_str.contains("list-unsubscribe-post"),
      "Signature should contain list-unsubscribe-post header"
    );
  }
}
