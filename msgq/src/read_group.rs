use std::fmt;

use aok::{OK, Void};
use fred::{
  interfaces::ClientLike,
  prelude::FredResult,
  types::{CustomCommand, Value},
};
use xkv::R;

use crate::{Conf, Parse, StreamItem, auto_new, parse_stream};

pub struct ReadGroup<P: Parse> {
  conf: Conf,
  parse: P,
  args: Vec<Value>,
}

impl<P: Parse + Clone + Send + Sync + 'static> ReadGroup<P> {
  pub fn new(parse: P, conf: Conf) -> Self {
    let count_str = conf.count.to_string();
    let claim_str = conf.claim_idle_ms.to_string();

    let mut args: Vec<Value> = vec![
      "GROUP".into(),
      conf.group.clone().into(),
      conf.consumer.clone().into(),
      "COUNT".into(),
      count_str.into(),
    ];

    if conf.block_ms > 0 {
      args.push("BLOCK".into());
      args.push(conf.block_ms.to_string().into());
    }

    args.extend(vec![
      "CLAIM".into(),
      claim_str.into(),
      "STREAMS".into(),
      conf.stream.clone().into(),
      ">".into(),
    ]);

    Self { conf, parse, args }
  }

  pub async fn run(&self) -> Void {
    let group = &self.conf.group;
    let stream = &self.conf.stream;

    loop {
      let li: FredResult<Vec<StreamItem>> = R
        .custom(
          CustomCommand::new("XREADGROUP", stream.as_bytes(), true),
          self.args.clone(),
        )
        .await
        .map(parse_stream);

      if let Some(li) = auto_new(stream, group, li).await? {
        if li.is_empty() {
          break;
        }

        let mut ing = Vec::with_capacity(li.len());

        for StreamItem { id, retry, kv, .. } in li {
          let parser = self.parse.clone();
          ing.push((
            id,
            retry,
            kv.clone(),
            tokio::spawn(async move { parser.run(&kv, retry).await }),
          ));
        }

        let mut id_li: Vec<String> = Vec::new();

        for (id, retry, kv, task) in ing {
          let err = match task.await {
            Ok(Err(err)) => err.to_string(),
            Err(err) => err.to_string(),
            _ => {
              id_li.push(id);
              continue;
            }
          };
          log::error!("{id} retry {retry} {err}");
          if retry > self.conf.max_retry {
            id_li.push(id);
            let parse = self.parse.clone();
            tokio::spawn(async move { xerr::log!(parse.on_error(kv, err).await) });
          }
        }
        if !id_li.is_empty() {
          crate::rm_id_li(&self.conf.stream, &self.conf.group, id_li).await?;
        }
      }
    }
    OK
  }
}

impl<P: Parse> fmt::Display for ReadGroup<P> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "{} {} {}",
      String::from_utf8_lossy(self.conf.stream.as_bytes()),
      self.conf.group,
      self.conf.consumer
    )
  }
}

impl<P: Parse> fmt::Debug for ReadGroup<P> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("ReadGroup")
      .field("conf", &self.conf)
      .finish()
  }
}
