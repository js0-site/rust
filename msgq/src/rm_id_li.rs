use fred::{interfaces::StreamsInterface, types::Key};
use xkv::R;

pub async fn rm_id_li(
  stream: &Key,
  group: &str,
  id_li: Vec<String>,
) -> fred::prelude::FredResult<()> {
  // wait for xackdel https://github.com/apache/kvrocks/pull/3275
  let p = R.pipeline();
  let _: () = p.xack(stream, group, id_li.clone()).await?;
  let _: () = p.xdel(stream, id_li).await?;
  p.last().await
}
