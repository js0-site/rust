use std::time::Duration;

use tokio::{spawn, task::JoinHandle, time::sleep};

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
