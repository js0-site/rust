use std::{
  fs::File,
  io::{Seek, SeekFrom, Write},
  path::PathBuf,
  sync::Arc,
};

use bytes::Bytes;
use crossfire::{AsyncRx, MTx, mpsc::List};
use ireq::{
  REQ,
  reqwest::{Url, header},
};
use log::warn;
use parking_lot::Mutex;
use tokio::{spawn, task::JoinHandle};

use crate::{ChunkLi, Result};

pub type Ing = JoinHandle<Result<()>>;

pub struct Runner {
  chunk_li: ChunkLi,
  ing: Arc<Mutex<Vec<Ing>>>,
}

impl Runner {
  pub fn new(
    size: u64,
    path: impl Into<PathBuf>,
    info_send: MTx<List<u64>>,
    data_recv: AsyncRx<List<(u64, Bytes)>>,
    on_end: impl FnOnce() + Send + 'static,
  ) -> Self {
    let path = path.into();
    let chunk_li = ChunkLi::new(size);

    let ing = Arc::new(Mutex::new(Vec::new()));

    let this = Self {
      chunk_li: chunk_li.clone(),
      ing: ing.clone(),
    };

    spawn(async move {
      let mut file = File::create(path)?;
      file.set_len(size)?;
      let mut downed = 0;
      while let Ok((begin, data)) = data_recv.recv().await {
        file.seek(SeekFrom::Start(begin))?;
        file.write_all(&data)?;
        if chunk_li.remove(begin).await {
          downed += data.len() as u64;
          info_send.send(downed)?;
          if downed == size {
            let mut ing = ing.lock();
            while let Some(i) = ing.pop() {
              i.abort();
            }
            on_end();
            return Ok::<(), crate::Error>(());
          }
        }
      }
      Ok::<(), crate::Error>(())
    });

    this
  }

  pub fn run(&mut self, url: Url, send: &MTx<List<(u64, Bytes)>>) {
    let send = send.clone();
    let chunk_li = self.chunk_li.clone();
    self.ing.lock().push(spawn(async move {
      while let Some((begin, end)) = chunk_li.get().await {
        match ireq::req(
          REQ
            .get(url.clone())
            .header(header::RANGE, format!("bytes={begin}-{end}")),
        )
        .await
        {
          Ok(data) => {
            // dbg!((&url.host().unwrap().to_string(), begin));
            send.send((begin, data))?;
          }
          Err(err) => {
            warn!("❌ {url} : {err}");
          }
        };
      }
      Ok(())
    }));
  }
}
