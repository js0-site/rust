#![cfg_attr(docsrs, feature(doc_cfg))]

use anyhow::Result;

pub trait Forward: Clone + Sync + Send + 'static {
  fn forward(&self, mail: &str) -> impl Future<Output = Result<Option<String>>> + Send;
  fn forward_set(&self, mail_li: &[String]) -> impl Future<Output = Result<Vec<String>>> + Send;
}
