#![allow(unused_imports)]
use std::{
  sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
  },
  time::Duration,
};

use aok::{OK, Void};
use log::info;
#[cfg(feature = "sync")]
use set_timer::set_timer;
use tokio::time::sleep;

#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
}

#[cfg(feature = "sync")]
#[tokio::test]
async fn test_set_timer() -> Void {
  let count = Arc::new(AtomicUsize::new(0));
  let count_clone = count.clone();

  let period = Duration::from_millis(100);
  let handle = set_timer(
    move || {
      count_clone.fetch_add(1, Ordering::SeqCst);
    },
    period,
  );

  sleep(Duration::from_millis(350)).await;

  handle.abort();

  let c = count.load(Ordering::SeqCst);
  info!("count: {}", c);
  assert!(c >= 3);
  OK
}

#[cfg(feature = "async")]
#[tokio::test]
async fn test_set_timer_async() -> Void {
  use set_timer::set_timer_async;
  let count = Arc::new(AtomicUsize::new(0));
  let count_clone = count.clone();

  let period = Duration::from_millis(100);
  let handle = set_timer_async(
    move || {
      let count_clone = count_clone.clone();
      async move {
        count_clone.fetch_add(1, Ordering::SeqCst);
      }
    },
    period,
  );

  sleep(Duration::from_millis(350)).await;

  handle.abort();

  let c = count.load(Ordering::SeqCst);
  info!("async count: {}", c);
  assert!(c >= 3);
  OK
}
