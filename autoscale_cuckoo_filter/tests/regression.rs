//! Regression tests for CuckooFilter.
//! CuckooFilter 回归测试

use std::collections::HashSet;

use autoscale_cuckoo_filter::{CuckooFilter, CuckooFilterBuilder};

/// Generate unique test data using HashSet.
/// 使用 HashSet 生成唯一测试数据
fn unique_u64(count: usize, seed: u64) -> Vec<u64> {
  fastrand::seed(seed);
  let mut set = HashSet::with_capacity(count);
  while set.len() < count {
    set.insert(fastrand::u64(..));
  }
  set.into_iter().collect()
}

/// Test basic add and contains operations.
#[test]
fn test_basic_add_contains() {
  let mut filter = CuckooFilter::<u64>::new(1000, 0.001);
  let data = unique_u64(100, 1);

  for item in &data {
    filter.add(item);
  }

  for item in &data {
    assert!(filter.contains(item), "Item {item} should be in filter");
  }

  assert_eq!(filter.len(), 100);
}

/// Test FPP within acceptable range.
#[test]
fn test_false_positive_rate() {
  let fpp = 0.01;
  let capacity = 1_000;
  let mut filter = CuckooFilter::<u64>::new(capacity, fpp);

  let data = unique_u64(capacity, 2);
  for item in &data {
    filter.add(item);
  }

  // Test with items not in filter (different seed)
  // 测试不在过滤器中的元素（不同种子）
  let test_data = unique_u64(1_000, 999);
  let mut false_positives = 0;
  for item in &test_data {
    if filter.contains(item) {
      false_positives += 1;
    }
  }

  let actual_fpp = false_positives as f64 / test_data.len() as f64;
  assert!(actual_fpp < fpp * 5.0, "FPP {actual_fpp} too high");
}

/// Test remove operation.
#[test]
fn test_remove() {
  let mut filter = CuckooFilter::<u64>::new(1000, 0.001);
  let data = unique_u64(100, 3);

  for item in &data {
    filter.add(item);
  }

  for item in &data[..50] {
    assert!(filter.remove(item), "Should remove {item}");
  }

  for item in &data[..50] {
    assert!(!filter.contains(item), "Item {item} should be removed");
  }

  for item in &data[50..] {
    assert!(filter.contains(item), "Item {item} should exist");
  }

  assert_eq!(filter.len(), 50);
}

/// Test duplicate add handling.
#[test]
fn test_duplicate_add() {
  let mut filter = CuckooFilter::<str>::new(1000, 0.001);

  filter.add("foo");
  filter.add("foo");
  filter.add("foo");

  assert_eq!(filter.len(), 3);
  assert!(filter.contains("foo"));

  filter.remove("foo");
  assert!(filter.contains("foo"));
  assert_eq!(filter.len(), 2);

  filter.remove("foo");
  filter.remove("foo");
  assert!(!filter.contains("foo"));
  assert_eq!(filter.len(), 0);
}

/// Test add_if_not_exist returns correct value.
/// 测试 add_if_not_exist 返回值
#[test]
fn test_add_if_not_exist() {
  let mut filter = CuckooFilter::<u64>::new(1000, 0.001);

  // First add returns false (not previously contained)
  // 首次添加返回 false（之前不存在）
  assert!(!filter.add_if_not_exist(&42));
  assert_eq!(filter.len(), 1);

  // Second add returns true (already contained)
  // 再次添加返回 true（已存在）
  assert!(filter.add_if_not_exist(&42));
  assert_eq!(filter.len(), 1);

  // Different item
  // 不同元素
  assert!(!filter.add_if_not_exist(&43));
  assert_eq!(filter.len(), 2);
}

/// Test automatic scaling.
#[test]
fn test_auto_scaling() {
  let initial_capacity = 100;
  let mut filter = CuckooFilter::<u64>::new(initial_capacity, 0.001);
  let data = unique_u64(1000, 4);

  let initial_cap = filter.capacity();

  for item in &data {
    filter.add(item);
  }

  assert!(
    filter.capacity() > initial_cap,
    "Capacity should grow: {} > {initial_cap}",
    filter.capacity()
  );

  for item in &data {
    assert!(
      filter.contains(item),
      "Item {item} should exist after scaling"
    );
  }
}

/// Test shrink_to_fit.
#[test]
fn test_shrink_to_fit() {
  let mut filter = CuckooFilter::<u64>::new(1000, 0.001);
  let data = unique_u64(100, 5);

  for item in &data {
    filter.add(item);
  }

  for item in &data {
    assert!(filter.contains(item), "Item {item} missing before shrink");
  }

  let bits_before = filter.bits();
  filter.shrink_to_fit();
  let bits_after = filter.bits();

  assert!(
    bits_after <= bits_before,
    "Bits should decrease: {bits_after} <= {bits_before}"
  );

  for item in &data {
    assert!(
      filter.contains(item),
      "Item {item} should exist after shrink"
    );
  }
}

/// Test with str type.
#[test]
fn test_str_type() {
  let mut filter = CuckooFilter::<str>::new(1000, 0.001);

  let items = ["hello", "world", "foo", "bar", "baz"];
  for item in &items {
    filter.add(*item);
  }

  for item in &items {
    assert!(filter.contains(*item));
  }

  assert!(!filter.contains("not_in_filter"));
}

/// Test with owned String type.
#[test]
fn test_owned_string() {
  let mut filter = CuckooFilter::<String>::new(1000, 0.001);

  let items: Vec<String> = vec!["hello".into(), "world".into(), "test".into()];

  for item in &items {
    filter.add(item);
  }

  for item in &items {
    assert!(filter.contains(item));
  }
}

/// Test empty filter.
#[test]
fn test_empty_filter() {
  let filter = CuckooFilter::<u64>::new(1000, 0.001);

  assert!(filter.is_empty());
  assert_eq!(filter.len(), 0);
  assert!(!filter.contains(&42));
}

/// Test clone.
#[test]
fn test_clone() {
  let mut filter = CuckooFilter::<u64>::new(1000, 0.001);
  let data = unique_u64(100, 6);

  for item in &data {
    filter.add(item);
  }

  let cloned = filter.clone();

  for item in &data {
    assert!(filter.contains(item));
    assert!(cloned.contains(item));
  }

  assert_eq!(filter.len(), cloned.len());
  assert_eq!(filter.capacity(), cloned.capacity());
}

/// Test builder configuration.
#[test]
fn test_builder_configuration() {
  let filter: CuckooFilter<u64> = CuckooFilterBuilder::new()
    .initial_capacity(500)
    .false_positive_probability(0.01)
    .entries_per_bucket(4)
    .max_kicks(256)
    .finish();

  assert_eq!(filter.false_positive_probability(), 0.01);
  assert_eq!(filter.entries_per_bucket(), 4);
  assert_eq!(filter.max_kicks(), 256);
}

/// Test deterministic behavior with seeded RNG.
#[test]
fn test_deterministic_with_seeded_rng() {
  let data = unique_u64(1000, 7);

  fastrand::seed(42);
  let mut filter1: CuckooFilter<u64> = CuckooFilterBuilder::new()
    .initial_capacity(100)
    .false_positive_probability(0.001)
    .finish();

  fastrand::seed(42);
  let mut filter2: CuckooFilter<u64> = CuckooFilterBuilder::new()
    .initial_capacity(100)
    .false_positive_probability(0.001)
    .finish();

  for item in &data {
    filter1.add(item);
    filter2.add(item);
  }

  assert_eq!(filter1.len(), filter2.len());
}

/// Stress test.
#[test]
fn test_large_scale() {
  let mut filter = CuckooFilter::<u64>::new(1_000, 0.01);
  let count = 5_000;
  let data = unique_u64(count, 8);

  for item in &data {
    filter.add(item);
  }

  assert_eq!(filter.len(), count);

  fastrand::seed(12345);
  for _ in 0..100 {
    let idx = fastrand::usize(0..count);
    assert!(filter.contains(&data[idx]));
  }
}

/// Test remove non-existent item.
#[test]
fn test_remove_nonexistent() {
  let mut filter = CuckooFilter::<u64>::new(1000, 0.001);

  filter.add(&1);
  filter.add(&2);

  assert!(!filter.remove(&999));
  assert_eq!(filter.len(), 2);
}

/// Test filter info methods.
#[test]
fn test_filter_info() {
  let mut filter = CuckooFilter::<u64>::new(1000, 0.001);
  let data = unique_u64(100, 9);

  assert!(filter.bits() > 0);
  assert!(filter.capacity() > 0);
  assert_eq!(filter.false_positive_probability(), 0.001);

  for item in &data {
    filter.add(item);
  }

  assert_eq!(filter.len(), 100);
  assert!(!filter.is_empty());
}

/// Test with various numeric types.
#[test]
fn test_numeric_types() {
  let mut filter_i32 = CuckooFilter::<i32>::new(100, 0.01);
  let mut filter_i64 = CuckooFilter::<i64>::new(100, 0.01);
  let mut filter_usize = CuckooFilter::<usize>::new(100, 0.01);

  for i in 0i32..50 {
    filter_i32.add(&i);
    filter_i64.add(&(i as i64));
    filter_usize.add(&(i as usize));
  }

  for i in 0i32..50 {
    assert!(filter_i32.contains(&i));
    assert!(filter_i64.contains(&(i as i64)));
    assert!(filter_usize.contains(&(i as usize)));
  }
}
