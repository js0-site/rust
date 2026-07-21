use std::fmt;

use aok::{OK, Void};
use fred::{
  interfaces::{ClientLike, StreamsInterface},
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

impl<P: Parse + Send + Sync + 'static> ReadGroup<P> {
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

        let ing = unsafe {
          async_scoped::TokioScope::scope_and_collect(|s| {
            for StreamItem { retry, kv, .. } in &li {
              s.spawn(self.parse.run(kv, *retry))
            }
          })
        }
        .await
        .1;

        let mut id_li: Vec<String> = Vec::new();

        let mut has_add = false;
        let p = R.pipeline();
        for (res, StreamItem { id, retry, kv, .. }) in ing.into_iter().zip(li) {
          let err = match res {
            Err(err) => err.to_string(),
            Ok(task) => match task {
              Ok(opt_kv) => {
                id_li.push(id);
                if let Some(new_kv) = opt_kv {
                  let _: () = p.xadd(&self.conf.stream, false, None, "*", new_kv).await?;
                  has_add = true;
                }
                continue;
              }
              Err(err) => err.to_string(),
            },
          };

          log::error!("{id} retry {retry} {err}");
          if retry > self.conf.max_retry {
            id_li.push(id);
            let task = self.parse.on_error(kv, err);
            if let Err(e) = task.await {
              log::error!("{e}");
            }
          }
        }

        if has_add {
          let _: () = p.last().await?;
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
