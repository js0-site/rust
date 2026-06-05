#![cfg_attr(docsrs, feature(doc_cfg))]

use std::env::{self, VarError};

use anyhow::Result;
use auth_trait::Auth;

#[derive(Debug, Clone)]
pub struct AuthEnv {
  pub user: String,
  pub password: String,
}

impl AuthEnv {
  pub fn load(prefix: &str) -> Result<Self, VarError> {
    Ok(Self {
      user: env::var(format!("{}_USER", prefix))?,
      password: env::var(format!("{}_PASSWORD", prefix))?,
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
