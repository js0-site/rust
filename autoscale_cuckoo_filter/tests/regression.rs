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
/// 测试基本添加与包含操作
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
/// 测试假阳性率在可接受范围内
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
/// 测试移除操作
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
/// 测试重复添加处理
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
/// 测试自动扩展
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
/// 测试收缩以适应当前元素
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
/// 测试 str 切片类型
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
/// 测试拥有所有权的 String 类型
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
/// 测试空过滤器
#[test]
fn test_empty_filter() {
  let filter = CuckooFilter::<u64>::new(1000, 0.001);

  assert!(filter.is_empty());
  assert_eq!(filter.len(), 0);
  assert!(!filter.contains(&42));
}

/// Test clone.
/// 测试克隆
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
/// 测试构建器配置
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
/// 测试使用确定性种子的随机数生成行为
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
/// 压力测试
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
/// 测试移除不存在的元素
#[test]
fn test_remove_nonexistent() {
  let mut filter = CuckooFilter::<u64>::new(1000, 0.001);

  filter.add(&1);
  filter.add(&2);

  assert!(!filter.remove(&999));
  assert_eq!(filter.len(), 2);
}

/// Test filter info methods.
/// 测试过滤器信息查询方法
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
/// 测试各种数值类型
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

/// Test different entries per bucket (e.g. 2 and 8).
/// 测试不同的每桶条目数（如 2 和 8）
#[test]
fn test_different_entries_per_bucket() {
  for entries in [2, 8] {
    let mut filter: CuckooFilter<u64> = CuckooFilterBuilder::new()
      .initial_capacity(200)
      .false_positive_probability(0.01)
      .entries_per_bucket(entries)
      .finish();

    let data = unique_u64(200, entries as u64);
    for item in &data {
      filter.add(item);
    }
    for item in &data {
      assert!(filter.contains(item));
    }
    for item in &data[..50] {
      assert!(filter.remove(item));
      assert!(!filter.contains(item));
    }
    for item in &data[50..] {
      assert!(filter.contains(item));
    }
  }
}

/// Test shrink_to_fit preserves all items including exceptional ones.
/// 测试 shrink_to_fit 保留包括异常区在内的所有元素
#[test]
fn test_shrink_to_fit_preserves_items() {
  let mut filter = CuckooFilter::<u64>::new(1000, 0.001);
  let data = unique_u64(500, 42);
  for item in &data {
    filter.add(item);
  }
  filter.shrink_to_fit();
  for item in &data {
    assert!(filter.contains(item), "Item {item} must exist after shrink");
  }
}

/// Test add_if_not_exist does not cause spurious growth on existing items.
/// 测试已存在元素调用 add_if_not_exist 时不产生虚假扩容
#[test]
fn test_no_spurious_grow_on_existing_items() {
  let mut filter = CuckooFilter::<u64>::new(100, 0.001);
  for i in 0..80 {
    filter.add_if_not_exist(&i);
  }
  let subfilter_count_before = filter.subfilter_count();

  // Re-insert existing items repeatedly
  // 重复插入已存在的元素
  for i in 0..80 {
    assert!(filter.add_if_not_exist(&i));
  }
  assert_eq!(
    filter.subfilter_count(),
    subfilter_count_before,
    "adding existing items must never trigger grow"
  );
}

/// Test edge cases: entries = 0 defensive fallback and entries >= 32 mask calculation.
/// 测试边界情况：entries = 0 的防御性处理与 entries >= 32 的掩码计算
#[test]
fn test_edge_cases_zero_entries_and_large_entries() {
  // entries = 0 must not divide by zero in builder or base
  let filter_zero: CuckooFilter<u64> = CuckooFilterBuilder::new()
    .initial_capacity(100)
    .entries_per_bucket(0)
    .finish();
  assert!(filter_zero.capacity() > 0);

  // entries = 32 must not overflow u32 shift
  let mut filter: CuckooFilter<u64> = CuckooFilterBuilder::new()
    .initial_capacity(200)
    .false_positive_probability(0.01)
    .entries_per_bucket(32)
    .finish();

  for i in 0..100 {
    filter.add_if_not_exist(&i);
  }
  for i in 0..100 {
    assert!(filter.contains(&i));
  }
}

/// Test contains_batch with direct value slice (&[u64]) without Vec<&T> allocation.
/// 测试使用直接数值切片 (&[u64]) 进行批量查询，无需额外的 Vec<&T> 堆分配
#[test]
fn test_batch_with_slice_of_values() {
  let mut filter = CuckooFilter::<u64>::new(100, 0.001);
  let items: Vec<u64> = (0..10).collect();
  for &item in &items[..5] {
    filter.add(&item);
  }

  // Pass &[u64] directly
  let results = filter.contains_batch(&items);
  assert_eq!(
    results,
    vec![
      true, true, true, true, true, false, false, false, false, false
    ]
  );
}
