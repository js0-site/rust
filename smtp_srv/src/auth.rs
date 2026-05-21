use std::time::Duration;

use aok::Result;
use auth_trait::Auth;
use fred::interfaces::KeysInterface;
use tokio::time::sleep;
use xkv::R;

use crate::r::{DOMAIN_HOST, USER};

#[derive(Clone)]
pub struct AuthKvrocks;

impl Auth for AuthKvrocks {
  async fn verify(&self, _host: &str, username: &str, password: &str) -> Result<Option<u64>> {
    let Some((prefix, domain)) = username.split_once('@') else {
      return Ok(None);
    };
    let domain = domain.to_lowercase();
    let prefix = prefix.to_lowercase();

    let domain_key = [DOMAIN_HOST, domain.as_bytes()].concat();
    let user_key = [USER, domain.as_bytes(), b":", prefix.as_bytes()].concat();

    let data: Vec<Option<Vec<u8>>> = R.mget((&domain_key[..], &user_key[..])).await?;
    if data.len() == 2
      && let (Some(host_id_bytes), Some(pass_bytes)) = (&data[0], &data[1])
      && pass_bytes.len() == 48
    {
      let salt = &pass_bytes[0..16];
      let hash = &pass_bytes[16..48];
      if password_::verify(password.as_bytes(), salt, hash) {
        return Ok(Some(intbin::bin_u64(host_id_bytes)));
      }
    } else {
      sleep(Duration::from_millis(100)).await;
    }
    Ok(None)
  }
}

