use std::time::Duration;

use aok::{OK, Void};
use dashmap::{DashMap, DashSet};
use expire_cache::Expire;

#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
}

#[tokio::test]
async fn test_map() -> Void {
  let cache: Expire<DashMap<_, _>> = Expire::new(1);
  cache.insert("key", "val");
  let val = cache.get("key");
  assert!(val.is_some());

  if let Some(val) = val {
    assert_eq!(&*val, &"val");
  }

  tokio::time::sleep(Duration::from_secs(3)).await;
  assert!(cache.get("key").is_none());
  OK
}

#[tokio::test]
async fn test_get_or_init() -> Void {
  use expire_cache::AsyncInit;

  struct TestInit;

  impl AsyncInit for TestInit {
    type Key = &'static str;
    type Val = &'static str;
    type Error = aok::Error;

    async fn init(_key: &Self::Key) -> Result<Self::Val, Self::Error> {
      Ok("val")
    }
  }

  let cache: Expire<DashMap<_, _>> = Expire::new(1);
  let val = cache.get_or_init_async::<TestInit>("key").await?;
  assert_eq!(&*val, &"val");
  OK
}

#[tokio::test]
async fn test_set() -> Void {
  let cache: Expire<DashSet<_>> = Expire::new(1);
  cache.insert("key", ());
  assert!(cache.get("key").is_some());

  tokio::time::sleep(Duration::from_secs(3)).await;
  assert!(cache.get("key").is_none());
  OK
}
