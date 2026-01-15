use std::{future::Future, time::Duration};

use tokio::{spawn, task::JoinHandle, time::sleep};

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
