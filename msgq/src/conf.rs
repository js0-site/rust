use fred::types::Key;

#[derive(Debug)]
pub struct Conf {
  pub stream: Key,
  pub group: String,
  pub consumer: String,
  pub count: u64,
  pub max_retry: u64,
  pub block_ms: u64,
  pub claim_idle_ms: u64,
}

impl Conf {
  pub fn new(
    stream: impl Into<Key>,
    group: impl Into<String>,
    consumer: impl Into<String>,
    block_sec: u64,
    claim_idle_sec: u64,
    count: u64,
    max_retry: u64,
  ) -> Self {
    Self {
      stream: stream.into(),
      group: group.into(),
      consumer: consumer.into(),
      count,
      max_retry,
      block_ms: block_sec * 1000,
      claim_idle_ms: claim_idle_sec * 1000,
    }
  }
}
