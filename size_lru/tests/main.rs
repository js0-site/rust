use std::time::Instant;

use aok::{OK, Void};
use log::trace;
use size_lru::Lhd;

#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
}

// Per-entry overhead added internally by Lhd
// Lhd 内部添加的每条目开销
const ENTRY_OVERHEAD: usize = 96;

#[test]
fn test_basic() -> Void {
  trace!("> LHD basic operations");

  let mut cache: Lhd<&str, &str> = Lhd::new(1024);

  cache.set("k1", "v1", 10);
  cache.set("k2", "v2", 20);
  cache.set("k3", "v3", 30);

  assert_eq!(cache.get(&"k1"), Some(&"v1"));
  assert_eq!(cache.get(&"k2"), Some(&"v2"));
  assert_eq!(cache.get(&"k3"), Some(&"v3"));
  assert_eq!(cache.get(&"none"), None);

  assert_eq!(cache.len(), 3);
  // size() now includes ENTRY_OVERHEAD per entry
  // size() 现在包含每条目的 ENTRY_OVERHEAD
  assert_eq!(cache.size(), 60 + 3 * ENTRY_OVERHEAD);

  cache.rm(&"k2");
  assert_eq!(cache.get(&"k2"), None);
  assert_eq!(cache.len(), 2);
  assert_eq!(cache.size(), 40 + 2 * ENTRY_OVERHEAD);

  trace!("LHD basic test passed");
  OK
}

#[test]
fn test_eviction() -> Void {
  trace!("> LHD eviction");

  // Max 100 bytes (user data) + overhead
  // 最大 100 字节（用户数据）+ 开销
  // 10 items * (10 + 96) = 1060 bytes total
  let mut cache: Lhd<i32, i32> = Lhd::new(10 * (10 + ENTRY_OVERHEAD));

  // Insert 10 items, 10 bytes each (+ overhead internally)
  // 插入 10 个条目，每个 10 字节（内部 + 开销）
  for i in 0..10 {
    cache.set(i, i * 10, 10);
  }
  assert_eq!(cache.size(), 10 * (10 + ENTRY_OVERHEAD));
  assert_eq!(cache.len(), 10);

  // Insert one more, should evict
  // 再插入一个，应触发淘汰
  cache.set(100, 1000, 10);
  assert!(cache.size() <= 10 * (10 + ENTRY_OVERHEAD));
  assert!(cache.len() <= 10);

  trace!("LHD eviction test passed");
  OK
}

#[test]
fn test_size_aware() -> Void {
  trace!("> LHD size-aware eviction");

  // Max capacity for 5 small items + 1 large item
  // 5 个小条目 + 1 个大条目的最大容量
  let mut cache: Lhd<i32, i32> = Lhd::new(5 * (10 + ENTRY_OVERHEAD) + 50 + ENTRY_OVERHEAD);

  // Insert small items with hits
  // 插入小条目并访问
  for i in 0..5 {
    cache.set(i, i, 10);
    // Hit multiple times
    // 多次访问
    for _ in 0..10 {
      cache.get(&i);
    }
  }

  // Insert large item
  // 插入大条目
  cache.set(100, 100, 50);

  // Large item should be evicted first (lower hit density)
  // 大条目应先被淘汰（命中密度低）
  // Ckp more operations, small frequently-hit items should survive
  // 更多操作后，小的高频访问条目应存活
  for _ in 0..100 {
    for i in 0..5 {
      cache.get(&i);
    }
  }

  // Small items should still exist
  // 小条目应仍存在
  let mut small_hits = 0;
  for i in 0..5 {
    if cache.get(&i).is_some() {
      small_hits += 1;
    }
  }
  assert!(small_hits >= 3, "Small items should survive: {small_hits}");

  trace!("LHD size-aware test passed");
  OK
}

#[test]
fn test_update() -> Void {
  trace!("> LHD update existing key");

  let mut cache: Lhd<&str, &str> = Lhd::new(1024);

  cache.set("k", "old", 10);
  assert_eq!(cache.get(&"k"), Some(&"old"));
  assert_eq!(cache.size(), 10 + ENTRY_OVERHEAD);

  cache.set("k", "new", 20);
  assert_eq!(cache.get(&"k"), Some(&"new"));
  assert_eq!(cache.size(), 20 + ENTRY_OVERHEAD);
  assert_eq!(cache.len(), 1);

  trace!("LHD update test passed");
  OK
}

#[test]
fn test_edge_cases() -> Void {
  trace!("> LHD edge cases");

  // Max 1 item with size 1 + overhead
  // 最大 1 个条目，大小 1 + 开销
  let mut cache: Lhd<i32, i32> = Lhd::new(1 + ENTRY_OVERHEAD);
  cache.set(1, 1, 1);
  cache.set(2, 2, 1);
  assert!(cache.len() <= 1);

  // Empty cache
  // 空缓存
  let cache: Lhd<i32, i32> = Lhd::new(100);
  assert!(cache.is_empty());
  assert_eq!(cache.len(), 0);
  assert_eq!(cache.size(), 0);

  // Remove non-existent
  // 删除不存在的
  let mut cache: Lhd<i32, i32> = Lhd::new(100);
  cache.rm(&999);

  trace!("LHD edge cases test passed");
  OK
}

/// Test O(1) complexity for get/set operations
/// 测试 get/set 操作的 O(1) 复杂度
#[test]
fn test_complexity() -> Void {
  trace!("> LHD O(1) complexity test");

  let sizes = [1000, 10000, 100000];
  let mut times = Vec::new();

  for &n in &sizes {
    // Large enough to avoid eviction during test
    // 足够大以避免测试期间淘汰
    let mut cache: Lhd<usize, usize> = Lhd::new(n * 100);

    // Warm up
    // 预热
    for i in 0..n {
      cache.set(i, i, 10);
    }

    let start = Instant::now();
    let ops = 10000;

    for i in 0..ops {
      cache.set(n + i, i, 10);
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

  trace!("LHD O(1) complexity verified");
  OK
}

/// Test rm operation
/// 测试删除操作
#[test]
fn test_rm() -> Void {
  trace!("> LHD rm operations");

  let mut cache: Lhd<i32, i32> = Lhd::new(10000);

  // Insert items
  // 插入条目
  for i in 0..10 {
    cache.set(i, i * 100, 10);
  }
  assert_eq!(cache.len(), 10);
  assert_eq!(cache.size(), 10 * (10 + ENTRY_OVERHEAD));

  // Remove middle item
  // 删除中间条目
  cache.rm(&5);
  assert_eq!(cache.get(&5), None);
  assert_eq!(cache.len(), 9);
  assert_eq!(cache.size(), 9 * (10 + ENTRY_OVERHEAD));

  // Remove first item
  // 删除第一个条目
  cache.rm(&0);
  assert_eq!(cache.get(&0), None);
  assert_eq!(cache.len(), 8);

  // Remove last item
  // 删除最后一个条目
  cache.rm(&9);
  assert_eq!(cache.get(&9), None);
  assert_eq!(cache.len(), 7);

  // Other items still accessible
  // 其他条目仍可访问
  assert_eq!(cache.get(&1), Some(&100));
  assert_eq!(cache.get(&8), Some(&800));

  // Remove non-existent (no-op)
  // 删除不存在的（无操作）
  cache.rm(&999);
  assert_eq!(cache.len(), 7);

  // Remove all remaining
  // 删除所有剩余
  for i in 1..9 {
    if i != 5 {
      cache.rm(&i);
    }
  }
  assert!(cache.is_empty());
  assert_eq!(cache.size(), 0);

  trace!("LHD rm test passed");
  OK
}

/// Test rm complexity O(1)
/// 测试删除复杂度 O(1)
#[test]
fn test_rm_complexity() -> Void {
  trace!("> LHD rm O(1) complexity test");

  let sizes = [1000, 10000, 50000];
  let mut times = Vec::new();

  for &n in &sizes {
    let mut cache: Lhd<usize, usize> = Lhd::new(n * 100);

    // Fill cache
    // 填满缓存
    for i in 0..n {
      cache.set(i, i, 10);
    }

    let start = Instant::now();
    let ops = 5000;

    // Remove items
    // 删除条目
    for i in 0..ops {
      cache.rm(&i);
    }

    let elapsed = start.elapsed();
    let per_op = elapsed.as_nanos() / ops as u128;
    times.push(per_op);
    trace!("  n={n}: {per_op} ns/op (rm)");
  }

  // O(1) means time per op should be roughly constant
  // O(1) 意味着每次操作时间应大致恒定
  // Allow 20x variance for HashMap resize and cache effects
  // 允许 20 倍方差用于 HashMap 调整和缓存效应
  let max_ratio = times.iter().max().unwrap() / times.iter().min().unwrap().max(&1);
  assert!(max_ratio < 20, "Rm ratio {max_ratio} too high for O(1)");

  trace!("LHD rm O(1) verified");
  OK
}

/// Test eviction complexity O(SAMPLES=32)
/// 测试淘汰复杂度 O(SAMPLES=32)
#[test]
fn test_eviction_complexity() -> Void {
  trace!("> LHD eviction complexity test");

  let sizes = [1000, 10000, 50000];
  let mut times = Vec::new();

  for &n in &sizes {
    // Small cache to force eviction
    // 小缓存以强制淘汰
    let mut cache: Lhd<usize, usize> = Lhd::new(n * 10);

    // Fill cache
    // 填满缓存
    for i in 0..n {
      cache.set(i, i, 10);
    }

    let start = Instant::now();
    let ops = 5000;

    // Each set triggers eviction
    // 每次 set 触发淘汰
    for i in 0..ops {
      cache.set(n + i, i, 10);
    }

    let elapsed = start.elapsed();
    let per_op = elapsed.as_nanos() / ops as u128;
    times.push(per_op);
    trace!("  n={n}: {per_op} ns/op (with eviction)");
  }

  // Eviction is O(32) regardless of cache size
  // 淘汰是 O(32)，与缓存大小无关
  let max_ratio = times.iter().max().unwrap() / times.iter().min().unwrap().max(&1);
  assert!(
    max_ratio < 5,
    "Eviction ratio {max_ratio} too high for O(32)"
  );

  trace!("LHD eviction O(32) verified");
  OK
}

#[test]
fn test_on_rm_callback() -> Void {
  use std::{cell::RefCell, rc::Rc};

  use size_lru::OnRm;

  trace!("> LHD OnRm callback test");

  // Track removed keys
  // 追踪被删除的 key
  let removed: Rc<RefCell<Vec<i32>>> = Rc::new(RefCell::new(Vec::new()));

  struct Cb(Rc<RefCell<Vec<i32>>>);

  impl<V> OnRm<i32, Lhd<i32, V, Self>> for Cb {
    fn call(&mut self, key: &i32, cache: &Lhd<i32, V, Self>) {
      // Can still peek value before removal or eviction
      // 删除/淘汰前仍可用 peek 获取值
      let _ = cache.peek(key);
      self.0.borrow_mut().push(*key);
    }
  }

  let cb = Cb(removed.clone());
  // Only fit 2 entries
  // 只能放 2 个条目
  let mut cache: Lhd<i32, i32, Cb> = Lhd::with_on_rm(2 * (10 + ENTRY_OVERHEAD), cb);

  cache.set(1, 100, 10);
  cache.set(2, 200, 10);

  // Manual rm triggers callback
  // 手动删除触发回调
  cache.rm(&2);
  assert_eq!(removed.borrow().len(), 1);
  assert_eq!(removed.borrow()[0], 2);

  // Eviction triggers callback
  // 淘汰触发回调
  cache.set(3, 300, 10);
  cache.set(4, 400, 10);
  // At least one eviction happened
  // 至少发生一次淘汰
  assert!(removed.borrow().len() >= 2);

  trace!("LHD OnRm callback test passed");
  OK
}
