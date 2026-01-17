//! Tests for Pgm (no data ownership)
//! Pgm 测试（不持有数据）

use aok::{OK, Void};
use jdb_pgm::Pgm;
use log::trace;

#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
}

/// Search in sorted slice using Pgm prediction
/// 使用 Pgm 预测在已排序切片中搜索
fn search<K: jdb_pgm::Key>(pgm: &Pgm<K>, sorted: &[K], key: K) -> Option<usize> {
  let range = pgm.predict_range(key);
  if range.start >= sorted.len() {
    return None;
  }
  let end = range.end.min(sorted.len());
  match sorted[range.start..end].binary_search(&key) {
    Ok(pos) => Some(range.start + pos),
    Err(_) => None,
  }
}

#[test]
fn test_basic() -> Void {
  let sorted: Vec<u64> = (0..10_000).collect();
  let pgm = Pgm::new(&sorted, 32);

  assert_eq!(search(&pgm, &sorted, 0), Some(0));
  assert_eq!(search(&pgm, &sorted, 5000), Some(5000));
  assert_eq!(search(&pgm, &sorted, 9999), Some(9999));
  assert_eq!(search(&pgm, &sorted, 10000), None);

  trace!("basic passed");
  OK
}

#[test]
fn test_epsilon() -> Void {
  let sorted: Vec<u64> = (0..50_000).collect();

  for &eps in &[1usize, 4, 16, 32, 64, 128, 256] {
    let pgm = Pgm::new(&sorted, eps);
    assert!(pgm.segment_count() >= 1);

    for &k in &[0u64, 1000, 25000, 49999] {
      assert_eq!(
        search(&pgm, &sorted, k),
        Some(k as usize),
        "eps={eps}, key={k}"
      );
    }
  }

  trace!("epsilon passed");
  OK
}

#[test]
fn test_non_existent() -> Void {
  let sorted: Vec<u64> = (0..1000).step_by(2).collect();
  let pgm = Pgm::new(&sorted, 16);

  assert_eq!(search(&pgm, &sorted, 1), None);
  assert_eq!(search(&pgm, &sorted, 3), None);
  assert_eq!(search(&pgm, &sorted, 0), Some(0));
  assert_eq!(search(&pgm, &sorted, 2), Some(1));

  trace!("non_existent passed");
  OK
}

#[test]
fn test_single() -> Void {
  let sorted = vec![42u64];
  let pgm = Pgm::new(&sorted, 1);

  assert_eq!(search(&pgm, &sorted, 42), Some(0));
  assert_eq!(search(&pgm, &sorted, 41), None);
  assert_eq!(pgm.segment_count(), 1);

  trace!("single passed");
  OK
}

#[test]
fn test_duplicates() -> Void {
  let sorted = vec![1u64, 1, 1, 2, 2, 3, 3, 3, 3];
  let pgm = Pgm::new(&sorted, 1);

  assert!(search(&pgm, &sorted, 1).is_some());
  assert!(search(&pgm, &sorted, 2).is_some());
  assert!(search(&pgm, &sorted, 3).is_some());
  assert_eq!(search(&pgm, &sorted, 0), None);
  assert_eq!(search(&pgm, &sorted, 4), None);

  trace!("duplicates passed");
  OK
}

#[test]
fn test_sparse() -> Void {
  let sorted: Vec<u64> = vec![1, 100, 10000, 1000000, 100000000];
  let pgm = Pgm::new(&sorted, 4);

  assert_eq!(search(&pgm, &sorted, 1), Some(0));
  assert_eq!(search(&pgm, &sorted, 100), Some(1));
  assert_eq!(search(&pgm, &sorted, 10000), Some(2));
  assert_eq!(search(&pgm, &sorted, 50), None);

  trace!("sparse passed");
  OK
}

#[test]
fn test_negative() -> Void {
  let sorted: Vec<i64> = (-1000..1000).collect();
  let pgm = Pgm::new(&sorted, 32);

  assert_eq!(search(&pgm, &sorted, -1000i64), Some(0));
  assert_eq!(search(&pgm, &sorted, 0i64), Some(1000));
  assert_eq!(search(&pgm, &sorted, 999i64), Some(1999));
  assert_eq!(search(&pgm, &sorted, -1001i64), None);

  trace!("negative passed");
  OK
}

#[test]
fn test_predict() -> Void {
  let sorted: Vec<u64> = (0..10_000).collect();
  let pgm = Pgm::new(&sorted, 32);

  for &k in &[0u64, 100, 5000, 9999] {
    let pred = pgm.predict(k);
    let error = pred.abs_diff(k as usize);
    assert!(error <= 32, "key={k}, pred={pred}, error={error}");
  }

  trace!("predict passed");
  OK
}

#[test]
fn test_mem() -> Void {
  let sorted: Vec<u64> = (0..100_000).collect();
  let pgm = Pgm::new(&sorted, 32);

  let mem = pgm.mem_usage();
  assert!(mem > 0);
  assert!(pgm.segment_count() >= 1);
  assert!(pgm.avg_segment_size() > 0.0);

  trace!("mem={mem} bytes, segments={}", pgm.segment_count());
  OK
}

#[test]
fn test_segment_vs_epsilon() -> Void {
  let sorted: Vec<u64> = (0..100_000).collect();

  let pgm_small = Pgm::new(&sorted, 8);
  let pgm_large = Pgm::new(&sorted, 128);

  assert!(pgm_small.segment_count() >= pgm_large.segment_count());

  trace!(
    "segments: small={}, large={}",
    pgm_small.segment_count(),
    pgm_large.segment_count()
  );
  OK
}

#[test]
fn test_all_same() -> Void {
  let sorted = vec![42u64; 100];
  let pgm = Pgm::new(&sorted, 1);

  assert!(search(&pgm, &sorted, 42).is_some());
  assert_eq!(search(&pgm, &sorted, 41), None);

  trace!("all_same passed");
  OK
}

#[test]
fn test_large_values() -> Void {
  let base = u64::MAX - 1000;
  let sorted: Vec<u64> = (0..1000).map(|i| base + i).collect();
  let pgm = Pgm::new(&sorted, 16);

  assert_eq!(search(&pgm, &sorted, base), Some(0));
  assert_eq!(search(&pgm, &sorted, base + 500), Some(500));
  assert_eq!(search(&pgm, &sorted, base - 1), None);

  trace!("large_values passed");
  OK
}

#[test]
fn test_quadratic() -> Void {
  let sorted: Vec<u64> = (0..1000u64).map(|i| i * i).collect();
  let pgm = Pgm::new(&sorted, 16);

  for (i, &k) in sorted.iter().enumerate() {
    assert_eq!(search(&pgm, &sorted, k), Some(i), "key={k}");
  }
  assert_eq!(search(&pgm, &sorted, 2), None);

  trace!("quadratic passed");
  OK
}

#[test]
fn test_random() -> Void {
  use rand::{Rng, SeedableRng, rngs::StdRng};

  let mut rng = StdRng::seed_from_u64(12345);
  let mut sorted: Vec<u64> = (0..10_000)
    .map(|_| rng.random_range(0..1_000_000))
    .collect();
  sorted.sort();
  sorted.dedup();

  let pgm = Pgm::new(&sorted, 32);

  for (i, &k) in sorted.iter().enumerate() {
    assert_eq!(search(&pgm, &sorted, k), Some(i), "key={k}");
  }

  trace!("random passed, n={}", sorted.len());
  OK
}
