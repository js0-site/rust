#![cfg_attr(docsrs, feature(doc_cfg))]

use std::ops::Deref;

use ed25519_dalek::SigningKey;

#[derive(Debug)]
pub struct Sk {
  hasher: blake3::Hasher,
}

#[derive(Debug)]
pub struct Dkim {
  pub sk: SigningKey,
}

impl Deref for Dkim {
  type Target = SigningKey;
  fn deref(&self) -> &Self::Target {
    &self.sk
  }
}

impl Dkim {
  #[cfg(feature = "pk")]
  pub fn txt(&self) -> String {
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    let pk = STANDARD.encode(self.sk.verifying_key());
    format!("v=DKIM1;k=ed25519;p={}", pk)
  }
}

impl Sk {
  pub fn new(sk: impl AsRef<[u8]>) -> Self {
    let mut hasher = blake3::Hasher::new();
    hasher.update(sk.as_ref());
    Self { hasher }
  }
  pub fn dkim(&self, selector: impl AsRef<str>, domain: impl AsRef<str>) -> Dkim {
    let selector = selector.as_ref();
    let domain = domain.as_ref();

    let mut hasher = self.hasher.clone();
    hasher.update(selector.as_bytes());
    hasher.update(b".");
    hasher.update(domain.as_bytes());
    let hash = hasher.finalize();

    let sk = ed25519_dalek::SigningKey::from_bytes(hash.as_bytes());
    Dkim { sk }
  }
}
