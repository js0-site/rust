#![cfg_attr(docsrs, feature(doc_cfg))]

use anyhow::Result;
use auth_trait::Auth;

pub struct AuthEnv {
  pub user: String,
  pub password: String,
}

impl AuthEnv {
  pub fn load(prefix: &str) -> Result<Self, std::env::VarError> {
    Ok(Self {
      user: std::env::var(format!("{}_USER", prefix))?,
      password: std::env::var(format!("{}_PASSWORD", prefix))?,
    })
  }
}

impl Auth for AuthEnv {
  async fn verify(&self, _host: &str, username: &str, password: &str) -> Result<Option<u64>> {
    Ok(if username == self.user && password == self.password {
      Some(1)
    } else {
      None
    })
  }
}
