use std::time::Duration;

use aok::{OK, Void};
use expire_cache::Expire;
use papaya::{HashMap, HashSet};
use tokio::time::sleep;

#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
}

#[tokio::test]
async fn test_map() -> Void {
  let cache: Expire<HashMap<&str, &str>> = Expire::new(1);
  cache.insert("key", "val");
  let val = cache.get("key");
  assert!(val.is_some());

  if let Some(val) = val {
    assert_eq!(val, "val");
  }

  sleep(Duration::from_secs(3)).await;
  assert!(cache.get("key").is_none());
  OK
}

#[tokio::test]
async fn test_get_or_init() -> Void {
  let cache: Expire<HashMap<String, &'static str>> = Expire::new(1);
  async fn init() -> aok::Result<&'static str> {
    Ok("val")
  }
  let val: Result<_, aok::Error> = expire_cache::get_or_init_async!(cache, "key", init);
  assert_eq!(val?, "val");
  OK
}

#[tokio::test]
async fn test_set() -> Void {
  let cache: Expire<HashSet<&str>> = Expire::new(1);
  cache.insert("key", ());
  assert!(cache.get("key").is_some());

  sleep(Duration::from_secs(3)).await;
  assert!(cache.get("key").is_none());
  OK
}
