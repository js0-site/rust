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
  let cache: Expire<DashMap<String, String>> = Expire::new(1);
  cache.insert(&"key".to_string(), "val".to_string());
  assert_eq!(cache.get(&"key".to_string()), Some("val".to_string()));

  tokio::time::sleep(Duration::from_secs(3)).await;
  assert_eq!(cache.get(&"key".to_string()), None);
  OK
}

#[tokio::test]
async fn test_set() -> Void {
  let cache: Expire<DashSet<String>> = Expire::new(1);
  cache.insert(&"key".to_string(), ());
  assert!(cache.get(&"key".to_string()).is_some());

  tokio::time::sleep(Duration::from_secs(3)).await;
  assert!(cache.get(&"key".to_string()).is_none());
  OK
}
