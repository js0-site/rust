use aok::Result;

use crate::Answer;

pub trait Resolver {
  fn resolve<T: Send + Unpin + std::fmt::Debug + 'static>(
    &self,
    name: impl AsRef<str> + Send,
    record_type: impl AsRef<str> + Send,
    extract: impl Fn(&[Answer]) -> aok::Result<Option<T>> + Send + 'static + Clone,
  ) -> impl std::future::Future<Output = Result<Option<T>>> + Send;
}

pub struct Resolve;

impl Resolver for Resolve {
  async fn resolve<T: Send + Unpin + std::fmt::Debug + 'static>(
    &self,
    name: impl AsRef<str> + Send,
    record_type: impl AsRef<str> + Send,
    extract: impl Fn(&[Answer]) -> aok::Result<Option<T>> + Send + 'static + Clone,
  ) -> Result<Option<T>> {
    crate::resolve::resolve(name, record_type, extract).await
  }
}
