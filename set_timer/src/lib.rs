#![cfg_attr(docsrs, feature(doc_cfg))]

use std::{future::Future, time::Duration};

use tokio::{spawn, task::JoinHandle, time::sleep};

#[cfg(feature = "async")]
pub fn set_timer_async<F, Fut>(func: F, period: Duration) -> JoinHandle<()>
where
  F: Fn() -> Fut + Send + Sync + 'static,
  Fut: Future<Output = ()> + Send,
{
  spawn(async move {
    loop {
      func().await;
      sleep(period).await;
    }
  })
}

#[cfg(feature = "sync")]
pub fn set_timer<F>(func: F, period: Duration) -> JoinHandle<()>
where
  F: Fn() + Send + Sync + 'static,
{
  spawn(async move {
    loop {
      func();
      sleep(period).await;
    }
  })
}
