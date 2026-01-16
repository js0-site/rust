//! Tests for PgmData (with data ownership)
//! PgmData 测试（持有数据）

use aok::{OK, Void};
use jdb_pgm::PgmData;
use log::trace;

#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
}

#[test]
fn test_basic() -> Void {
  let data: Vec<u64> = (0..10_000).collect();
  let idx = PgmData::load(data, 32, true)?;

  assert_eq!(idx.get(0), Some(0));
  assert_eq!(idx.get(5000), Some(5000));
  assert_eq!(idx.get(9999), Some(9999));
  assert_eq!(idx.get(10000), None);

  trace!("basic passed");
  OK
}

#[test]
fn test_epsilon() -> Void {
  let data: Vec<u64> = (0..50_000).collect();

  for &eps in &[1usize, 4, 16, 32, 64, 128, 256] {
    let idx = PgmData::load(data.clone(), eps, true)?;
    assert!(idx.segment_count() >= 1);

    for &k in &[0u64, 1000, 25000, 49999] {
      assert_eq!(idx.get(k), Some(k as usize), "eps={eps}, key={k}");
    }
  }

  trace!("epsilon passed");
  OK
}

#[test]
fn test_non_existent() -> Void {
  let data: Vec<u64> = (0..1000).step_by(2).collect();
  let idx = PgmData::load(data, 16, true)?;

  assert_eq!(idx.get(1), None);
  assert_eq!(idx.get(3), None);
  assert_eq!(idx.get(0), Some(0));
  assert_eq!(idx.get(2), Some(1));

  trace!("non_existent passed");
  OK
}

#[test]
fn test_single() -> Void {
  let data = vec![42u64];
  let idx = PgmData::load(data, 1, true)?;

  assert_eq!(idx.get(42), Some(0));
  assert_eq!(idx.get(41), None);
  assert_eq!(idx.segment_count(), 1);

  trace!("single passed");
  OK
}

#[test]
fn test_batch() -> Void {
  let data: Vec<u64> = (0..10_000).collect();
  let idx = PgmData::load(data, 32, true)?;

  let keys: Vec<u64> = (0..100).collect();
  let results: Vec<_> = idx.get_many(keys.iter().copied()).collect();

  for (i, r) in results.iter().enumerate() {
    assert_eq!(*r, Some(i));
  }

  trace!("batch passed");
  OK
}

#[test]
fn test_count_hits() -> Void {
  let data: Vec<u64> = (0..10_000).collect();
  let idx = PgmData::load(data, 32, true)?;

  let keys: Vec<u64> = (0..100).collect();
  assert_eq!(idx.count_hits(keys.iter().copied()), 100);

  let mixed: Vec<u64> = (9990..10010).collect();
  assert_eq!(idx.count_hits(mixed.iter().copied()), 10);

  trace!("count_hits passed");
  OK
}

#[test]
fn test_stats() -> Void {
  let data: Vec<u64> = (0..10_000).collect();
  let idx = PgmData::load(data, 32, true)?;
  let stats = idx.stats();

  assert!(stats.segments >= 1);
  assert!(stats.avg_segment_size > 0.0);
  assert!(stats.memory_bytes > 0);

  trace!("stats passed");
  OK
}

#[test]
fn test_memory() -> Void {
  let data: Vec<u64> = (0..100_000).collect();
  let idx = PgmData::load(data, 32, true)?;

  let mem = idx.memory_usage();
  assert!(mem >= 800_000); // at least data size

  trace!("memory={mem} bytes");
  OK
}

#[test]
fn test_deref() -> Void {
  let data: Vec<u64> = (0..10_000).collect();
  let idx = PgmData::load(data, 32, true)?;

  // Access Pgm methods via Deref
  let pred = idx.predict(5000);
  let error = pred.abs_diff(5000);
  assert!(error <= 32);

  let (start, end) = idx.predict_range(5000);
  assert!(start <= 5000 && 5000 < end);

  trace!("deref passed");
  OK
}

#[test]
fn test_predict_pos() -> Void {
  let data: Vec<u64> = (0..10_000).collect();
  let idx = PgmData::load(data, 32, true)?;

  for &k in &[0u64, 100, 5000, 9999] {
    let pred = idx.predict_pos(k);
    let error = pred.abs_diff(k as usize);
    assert!(error <= 32, "key={k}, pred={pred}, error={error}");
  }

  trace!("predict_pos passed");
  OK
}

#[test]
fn test_negative() -> Void {
  let data: Vec<i64> = (-1000..1000).collect();
  let idx = PgmData::load(data, 32, true)?;

  assert_eq!(idx.get(-1000i64), Some(0));
  assert_eq!(idx.get(0i64), Some(1000));
  assert_eq!(idx.get(999i64), Some(1999));
  assert_eq!(idx.get(-1001i64), None);

  trace!("negative passed");
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

  let idx = PgmData::load(data.clone(), 32, true)?;

  for (i, &k) in data.iter().enumerate() {
    assert_eq!(idx.get(k), Some(i), "key={k}");
  }

  trace!("random passed, n={}", data.len());
  OK
}
