//! Tests for Pgm (no data ownership)
//! Pgm 测试（不持有数据）

use aok::{OK, Void};
use jdb_pgm::Pgm;
use log::trace;

#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
}

/// Search in data slice using Pgm prediction
/// 使用 Pgm 预测在数据切片中搜索
fn search<K: jdb_pgm::Key>(pgm: &Pgm<K>, data: &[K], key: K) -> Option<usize> {
  let (start, end) = pgm.predict_range(key);
  if start >= data.len() {
    return None;
  }
  let end = end.min(data.len());
  match data[start..end].binary_search(&key) {
    Ok(pos) => Some(start + pos),
    Err(_) => None,
  }
}

#[test]
fn test_basic() -> Void {
  let data: Vec<u64> = (0..10_000).collect();
  let pgm = Pgm::new(&data, 32, true)?;

  assert_eq!(search(&pgm, &data, 0), Some(0));
  assert_eq!(search(&pgm, &data, 5000), Some(5000));
  assert_eq!(search(&pgm, &data, 9999), Some(9999));
  assert_eq!(search(&pgm, &data, 10000), None);

  trace!("basic passed");
  OK
}

#[test]
fn test_epsilon() -> Void {
  let data: Vec<u64> = (0..50_000).collect();

  for &eps in &[1usize, 4, 16, 32, 64, 128, 256] {
    let pgm = Pgm::new(&data, eps, true)?;
    assert!(pgm.segment_count() >= 1);

    for &k in &[0u64, 1000, 25000, 49999] {
      assert_eq!(
        search(&pgm, &data, k),
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
  let data: Vec<u64> = (0..1000).step_by(2).collect();
  let pgm = Pgm::new(&data, 16, true)?;

  assert_eq!(search(&pgm, &data, 1), None);
  assert_eq!(search(&pgm, &data, 3), None);
  assert_eq!(search(&pgm, &data, 0), Some(0));
  assert_eq!(search(&pgm, &data, 2), Some(1));

  trace!("non_existent passed");
  OK
}

#[test]
fn test_single() -> Void {
  let data = vec![42u64];
  let pgm = Pgm::new(&data, 1, true)?;

  assert_eq!(search(&pgm, &data, 42), Some(0));
  assert_eq!(search(&pgm, &data, 41), None);
  assert_eq!(pgm.segment_count(), 1);

  trace!("single passed");
  OK
}

#[test]
fn test_duplicates() -> Void {
  let data = vec![1u64, 1, 1, 2, 2, 3, 3, 3, 3];
  let pgm = Pgm::new(&data, 1, true)?;

  assert!(search(&pgm, &data, 1).is_some());
  assert!(search(&pgm, &data, 2).is_some());
  assert!(search(&pgm, &data, 3).is_some());
  assert_eq!(search(&pgm, &data, 0), None);
  assert_eq!(search(&pgm, &data, 4), None);

  trace!("duplicates passed");
  OK
}

#[test]
fn test_sparse() -> Void {
  let data: Vec<u64> = vec![1, 100, 10000, 1000000, 100000000];
  let pgm = Pgm::new(&data, 4, true)?;

  assert_eq!(search(&pgm, &data, 1), Some(0));
  assert_eq!(search(&pgm, &data, 100), Some(1));
  assert_eq!(search(&pgm, &data, 10000), Some(2));
  assert_eq!(search(&pgm, &data, 50), None);

  trace!("sparse passed");
  OK
}

#[test]
fn test_negative() -> Void {
  let data: Vec<i64> = (-1000..1000).collect();
  let pgm = Pgm::new(&data, 32, true)?;

  assert_eq!(search(&pgm, &data, -1000i64), Some(0));
  assert_eq!(search(&pgm, &data, 0i64), Some(1000));
  assert_eq!(search(&pgm, &data, 999i64), Some(1999));
  assert_eq!(search(&pgm, &data, -1001i64), None);

  trace!("negative passed");
  OK
}

#[test]
fn test_predict() -> Void {
  let data: Vec<u64> = (0..10_000).collect();
  let pgm = Pgm::new(&data, 32, true)?;

  for &k in &[0u64, 100, 5000, 9999] {
    let pred = pgm.predict(k);
    let error = pred.abs_diff(k as usize);
    assert!(error <= 32, "key={k}, pred={pred}, error={error}");
  }

  trace!("predict passed");
  OK
}

#[test]
fn test_predict_range() -> Void {
  let data: Vec<u64> = (0..10_000).collect();
  let pgm = Pgm::new(&data, 32, true)?;

  for &k in &[0u64, 100, 5000, 9999] {
    let (start, end) = pgm.predict_range(k);
    let actual = k as usize;
    assert!(
      start <= actual && actual < end,
      "key={k}, range=[{start}, {end})"
    );
  }

  trace!("predict_range passed");
  OK
}

#[test]
fn test_mem() -> Void {
  let data: Vec<u64> = (0..100_000).collect();
  let pgm = Pgm::new(&data, 32, true)?;

  let mem = pgm.mem_usage();
  assert!(mem > 0);
  assert!(pgm.segment_count() >= 1);
  assert!(pgm.avg_segment_size() > 0.0);

  trace!("mem={mem} bytes, segments={}", pgm.segment_count());
  OK
}

#[test]
fn test_segment_vs_epsilon() -> Void {
  let data: Vec<u64> = (0..100_000).collect();

  let pgm_small = Pgm::new(&data, 8, true)?;
  let pgm_large = Pgm::new(&data, 128, true)?;

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
  let data = vec![42u64; 100];
  let pgm = Pgm::new(&data, 1, true)?;

  assert!(search(&pgm, &data, 42).is_some());
  assert_eq!(search(&pgm, &data, 41), None);

  trace!("all_same passed");
  OK
}

#[test]
fn test_large_values() -> Void {
  let base = u64::MAX - 1000;
  let data: Vec<u64> = (0..1000).map(|i| base + i).collect();
  let pgm = Pgm::new(&data, 16, true)?;

  assert_eq!(search(&pgm, &data, base), Some(0));
  assert_eq!(search(&pgm, &data, base + 500), Some(500));
  assert_eq!(search(&pgm, &data, base - 1), None);

  trace!("large_values passed");
  OK
}

#[test]
fn test_quadratic() -> Void {
  let data: Vec<u64> = (0..1000u64).map(|i| i * i).collect();
  let pgm = Pgm::new(&data, 16, true)?;

  for (i, &k) in data.iter().enumerate() {
    assert_eq!(search(&pgm, &data, k), Some(i), "key={k}");
  }
  assert_eq!(search(&pgm, &data, 2), None);

  trace!("quadratic passed");
  OK
}

#[test]
fn test_random() -> Void {
  use rand::{Rng, SeedableRng, rngs::StdRng};

  let mut rng = StdRng::seed_from_u64(12345);
  let mut data: Vec<u64> = (0..10_000)
    .map(|_| rng.random_range(0..1_000_000))
    .collect();
  data.sort();
  data.dedup();

  let pgm = Pgm::new(&data, 32, true)?;

  for (i, &k) in data.iter().enumerate() {
    assert_eq!(search(&pgm, &data, k), Some(i), "key={k}");
  }

  trace!("random passed, n={}", data.len());
  OK
}
