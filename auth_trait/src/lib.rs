#![cfg_attr(docsrs, feature(doc_cfg))]

use std::future::Future;

use anyhow::Result;

pub trait Auth: Clone + Send + Sync + 'static {
  fn verify(
    &self,
    host: &str,
    username: &str,
    password: &str,
  ) -> impl Future<Output = Result<Option<u64>>> + Send;
}
