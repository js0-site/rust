#![cfg_attr(docsrs, feature(doc_cfg))]

use std::ops::Deref;

use rand_chacha::ChaCha20Rng;
use rsa::{RsaPrivateKey, rand_core::SeedableRng};

#[derive(Debug)]
pub struct Sk {
  hasher: blake3::Hasher,
}

#[derive(Debug)]
pub struct Dkim {
  pub sk: RsaPrivateKey,
}

impl Deref for Dkim {
  type Target = RsaPrivateKey;
  fn deref(&self) -> &Self::Target {
    &self.sk
  }
}

impl Dkim {
  #[cfg(feature = "pk")]
  pub fn txt(&self) -> String {
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    use rsa::pkcs8::EncodePublicKey;

    let public_key = self.sk.to_public_key();
    // 使用 SPKI (SubjectPublicKeyInfo) 格式，这是 DKIM 标准要求的格式
    let der = public_key
      .to_public_key_der()
      .expect("Failed to encode RSA public key");
    let pk = STANDARD.encode(der.as_bytes());
    format!("v=DKIM1; k=rsa; p={}", pk)
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

    // 使用 blake3 哈希作为 RNG 种子生成确定性的 RSA 密钥

    let mut rng = ChaCha20Rng::from_seed(*hash.as_bytes());

    let bits = 2048;
    let sk = RsaPrivateKey::new(&mut rng, bits).expect("Failed to generate RSA key");

    Dkim { sk }
  }
}
