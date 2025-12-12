use aok::{OK, Void};
use log::info;
use restart_signal::restart_signal;

#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
}

async fn background_task() {
  let mut counter = 0;
  loop {
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    counter += 1;
    info!("Background task running, count: {}", counter);
    if counter > 1000 {
      panic!("Background task exceeded max iterations without signal");
    }
  }
}

#[tokio::test]
async fn test_restart_signal() -> Void {
  let handle = tokio::spawn(async {
    tokio::select! {
      result = restart_signal() => {
        match result {
          Ok(signal) => {
            info!("Received signal: {}, stopping background task", signal);
          }
          Err(e) => {
            panic!("Failed to receive signal: {}", e);
          }
        }
      }
      _ = background_task() => {
        unreachable!("Background task should be interrupted by signal");
      }
    }
  });

  info!("pid {}", std::process::id());
  // unsafe {
  //   libc::raise(SIGTERM);
  // }

  handle.await.unwrap();
  OK
}
