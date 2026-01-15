#![cfg_attr(docsrs, feature(doc_cfg))]

use argon2::{Algorithm, Argon2, Params, Version};

#[static_init::dynamic]
static ARGON2: Argon2<'static> = Argon2::new(
  Algorithm::Argon2id,
  Version::V0x13,
  Params::new(65536, 3, 1, Some(32)).unwrap(),
);

pub type SALT = [u8; 16];
pub type HASH = [u8; 32];

pub fn hash_with_salt(password: impl AsRef<[u8]>, salt: impl AsRef<[u8]>) -> HASH {
  let mut hash = [0u8; 32];
  ARGON2
    .hash_password_into(password.as_ref(), salt.as_ref(), &mut hash)
    .unwrap();
  hash
}

pub fn hash(password: impl AsRef<[u8]>) -> (SALT, HASH) {
  let salt = rand::random();
  (salt, hash_with_salt(password, salt))
}

pub fn verify(password: impl AsRef<[u8]>, salt: impl AsRef<[u8]>, hash: impl AsRef<[u8]>) -> bool {
  hash.as_ref() == hash_with_salt(password.as_ref(), salt.as_ref()).as_ref()
}
