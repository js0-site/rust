use std::path::PathBuf;

use crossfire::{
  AsyncRx,
  mpsc::{List, unbounded_async},
};
use ireq::{
  REQ,
  reqwest::{IntoUrl, Url},
};
use log::warn;
use tokio::{spawn, task::JoinHandle};

mod chunk_li;
use chunk_li::ChunkLi;
mod error;
pub use error::{Error, Result};
mod runner;
use runner::Runner;

pub async fn meta(url: impl IntoUrl) -> Result<(u64, Url)> {
  let res = REQ
    .get(url)
    .header("User-Agent", "curl/8.4.0")
    .send()
    .await?;
  let status = res.status();
  if ireq::SUCCESS_STATUS.contains(&status) {
    return Ok((res.content_length().unwrap_or(0), res.url().clone()));
  }
  Err(Error::HttpResponse(status))
}

pub async fn down<U: IntoUrl>(
  url_li: impl IntoIterator<Item = U>,
  to_path: impl Into<PathBuf>,
) -> Result<AsyncRx<List<u64>>> {
  let (send, recv) = unbounded_async();

  let mut ing: Vec<JoinHandle<Result<()>>> = Vec::new();
  for i in url_li.into_iter() {
    let i = i.into_url()?;
    let send = send.clone();
    ing.push(spawn(async move {
      match meta(i.clone()).await {
        Err(err) => warn!("{} : {err}", i),
        Ok((size, url)) => {
          if size > 0 {
            send.send((url, size))?;
            drop(send);
          } else {
            warn!("{} filesize = 0", i);
          }
        }
      }
      Ok(())
    }));
  }

  drop(send);
  let (data_send, data_recv) = unbounded_async();

  let (info_send, info_recv) = unbounded_async();
  if let Ok((first_url, filesize)) = recv.recv().await {
    info_send.send(filesize)?;
    let mut runner = Runner::new(filesize, to_path, info_send, data_recv, || {
      for i in ing {
        i.abort();
      }
    });
    runner.run(first_url, &data_send);

    spawn(async move {
      while let Ok((url, size)) = recv.recv().await {
        if filesize == size {
          runner.run(url, &data_send);
        } else {
          warn!("{} filesize != {}", url, size);
        }
      }
      drop(data_send);
    });
  }

  Ok(info_recv)
}
