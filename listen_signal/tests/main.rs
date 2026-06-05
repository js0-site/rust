use std::process;

use aok::{OK, Void};
use futures::StreamExt;
use listen_signal::wait;
use log::info;
use tokio::time::{Duration, sleep};

#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
}

async fn background_task() {
  let mut counter = 0;
  loop {
    sleep(Duration::from_secs(1)).await;
    counter += 1;
    info!("Background task running, count: {}", counter);
    if counter > 2 {
      break;
    }
  }
}

#[tokio::test]
async fn test_listen_signal() -> Void {
  let handle = tokio::spawn(async {
    let mut signals = wait(listen_signal::SINGAL_LI);
    tokio::select! {
      signal = signals.next() => {
        info!("Received signal: {:?}, stopping background task", signal);
      }
      _ = background_task() => {
        unreachable!("Background task should be interrupted by signal");
      }
    }
  });

  tokio::spawn(async {
    sleep(Duration::from_millis(3000)).await;
    info!("Sending SIGHUP signal to pid {}", process::id());
    unsafe {
      libc::raise(libc::SIGHUP);
    }
  });

  let _signal = handle.await;
  OK
}
