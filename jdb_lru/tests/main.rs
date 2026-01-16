use aok::{OK, Void};
use jdb_lru::{Cache, Lru, NoCache};
use log::trace;

#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
}

#[test]
fn test_lru_basic() -> Void {
  trace!("> LRU basic operations");

  let mut cache = Lru::new(3);

  cache.set("k1".to_string(), "v1");
  cache.set("k2".to_string(), "v2");
  cache.set("k3".to_string(), "v3");

  assert_eq!(cache.get(&"k1".to_string()), Some(&"v1"));
  assert_eq!(cache.get(&"k2".to_string()), Some(&"v2"));
  assert_eq!(cache.get(&"k3".to_string()), Some(&"v3"));
  assert_eq!(cache.get(&"nonexistent".to_string()), None);

  cache.rm(&"k2".to_string());
  assert_eq!(cache.get(&"k2".to_string()), None);

  trace!("LRU basic test passed");
  OK
}

#[test]
fn test_lru_eviction() -> Void {
  trace!("> LRU eviction");

  let mut cache = Lru::new(2);

  cache.set(1, "a");
  cache.set(2, "b");

  // Access 1 to make it recent
  // 访问 1 使其成为最近使用
  assert_eq!(cache.get(&1), Some(&"a"));

  // Add 3, should evict 2 (least recent)
  // 添加 3，应淘汰 2（最久未使用）
  cache.set(3, "c");

  assert_eq!(cache.get(&1), Some(&"a"));
  assert_eq!(cache.get(&2), None);
  assert_eq!(cache.get(&3), Some(&"c"));

  trace!("LRU eviction test passed");
  OK
}

#[test]
fn test_lru_update() -> Void {
  trace!("> LRU update existing key");

  let mut cache = Lru::new(2);

  cache.set("k", "old");
  assert_eq!(cache.get(&"k"), Some(&"old"));

  cache.set("k", "new");
  assert_eq!(cache.get(&"k"), Some(&"new"));

  trace!("LRU update test passed");
  OK
}

#[test]
fn test_nocache() -> Void {
  trace!("> NoCache operations");

  let mut cache = NoCache;

  <NoCache as Cache<i32, &str>>::set(&mut cache, 1, "v");
  assert!(<NoCache as Cache<i32, &str>>::get(&mut cache, &1).is_none());
  <NoCache as Cache<i32, &str>>::rm(&mut cache, &1);
  assert!(<NoCache as Cache<i32, &str>>::get(&mut cache, &1).is_none());

  trace!("NoCache test passed");
  OK
}

#[test]
fn test_lru_edge_cases() -> Void {
  trace!("> LRU edge cases");

  // Capacity 1
  // 容量 1
  let mut cache = Lru::new(1);
  cache.set(1, "a");
  cache.set(2, "b");
  assert_eq!(cache.get(&1), None);
  assert_eq!(cache.get(&2), Some(&"b"));

  // Capacity 0 -> 1
  // 容量 0 -> 1
  let mut cache = Lru::new(0);
  cache.set("k", "v");
  assert_eq!(cache.get(&"k"), Some(&"v"));

  // Remove non-existent
  // 删除不存在的
  let mut cache: Lru<i32, i32> = Lru::new(3);
  cache.rm(&999);

  trace!("LRU edge cases test passed");
  OK
}

/// Test O(1) complexity for LRU operations
/// 测试 LRU 操作的 O(1) 复杂度
#[test]
fn test_lru_complexity() -> Void {
  trace!("> LRU O(1) complexity test");

  use std::time::Instant;

  // Test with different sizes, time should be similar
  // 测试不同大小，时间应相似
  let sizes = [1000, 10000, 100000];
  let mut times = Vec::new();

  for &n in &sizes {
    let mut cache = Lru::new(n);

    // Warm up
    // 预热
    for i in 0..n {
      cache.set(i, i);
    }

    let start = Instant::now();
    let ops = 10000;

    for i in 0..ops {
      cache.set(n + i, i);
      cache.get(&(i % n));
    }

    let elapsed = start.elapsed();
    let per_op = elapsed.as_nanos() / (ops * 2) as u128;
    times.push(per_op);
    trace!("  n={n}: {per_op} ns/op");
  }

  // O(1) means time per op should be roughly constant
  // O(1) 意味着每次操作时间应大致恒定
  // Allow 5x variance for noise
  // 允许 5 倍方差用于噪声
  let max_ratio = times.iter().max().unwrap() / times.iter().min().unwrap().max(&1);
  assert!(max_ratio < 5, "Time ratio {max_ratio} too high for O(1)");

  trace!("LRU O(1) complexity verified");
  OK
}
