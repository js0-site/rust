mod auto_new;

pub use auto_new::auto_new;
use bytes::Bytes;
use fred::types::streams::XReadValue;
mod conf;
pub use conf::Conf;
mod read_group;
pub use read_group::ReadGroup;
mod parse_stream;
pub use parse_stream::parse_stream;
mod rm_id_li;
pub use rm_id_li::rm_id_li;

pub type StreamData = XReadValue<String, Bytes, Bytes>;
pub type Kv = Vec<(Bytes, Bytes)>;

pub struct StreamItem {
  pub id: String,
  pub retry: u64,
  pub idle_ms: u64,
  pub kv: Kv,
}

pub trait Parse {
  fn run(&self, kv: &Kv, retry: u64) -> impl Future<Output = aok::Result<Option<Kv>>> + Send;
  fn on_error(&self, kv: Kv, error: String) -> impl Future<Output = aok::Void> + Send;
}
