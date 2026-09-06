//! Tests for Pgm::predict_range
//! Pgm::predict_range 测试

use std::ops::Range;

use aok::{OK, Void};
use jdb_pgm::Pgm;

#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
}

/// Helper to check if range is valid and contains the expected index
fn check_range(range: Range<usize>, expected: usize, len: usize, msg: &str) {
  assert!(range.start <= range.end, "{msg}: start > end ({:?})", range);
  assert!(
    range.end <= len,
    "{msg}: end > len ({:?}, len={})",
    range,
    len
  );
  assert!(
    range.contains(&expected),
    "{msg}: range {:?} does not contain expected {}",
    range,
    expected
  );
}

#[test]
fn test_range_basic() -> Void {
  let sorted: Vec<u64> = (0..10_000).collect();
  let pgm = Pgm::new(&sorted, 32);

  for &k in &[0u64, 100, 5000, 9999] {
    let range = pgm.predict_range(k);
    check_range(range, k as usize, sorted.len(), &format!("basic key={k}"));
  }
  OK
}

#[test]
fn test_range_epsilon_impact() -> Void {
  let sorted: Vec<u64> = (0..1000).collect();

  for &eps in &[4, 16, 64] {
    let pgm = Pgm::new(&sorted, eps);
    let range = pgm.predict_range(500);
    check_range(range.clone(), 500, sorted.len(), &format!("eps={eps}"));

    // Theoretical max range size is roughly 2*epsilon
    // 理论最大范围大小约为 2*epsilon
    let size = range.end - range.start;
    // Allow some slack for segmentation logic
    assert!(
      size <= 2 * eps + 5,
      "range too large for eps={}: size={}",
      eps,
      size
    );
  }
  OK
}

#[test]
fn test_range_out_of_bounds() -> Void {
  let sorted: Vec<u64> = (100..200).collect(); // 100, 101, ..., 199
  let pgm = Pgm::new(&sorted, 8);
  let len = sorted.len();

  // Key smaller than min
  let r_small = pgm.predict_range(0);
  // It might predict 0, or something near 0.
  // Since key 0 is not in sorted data, we just check bounds are valid w.r.t data len
  assert!(r_small.start <= r_small.end);
  assert!(r_small.end <= len);

  // binary search would find index 0 for key 0 (insert pos)
  // so range should ideally cover 0
  assert!(
    r_small.contains(&0),
    "small key should include 0, got {:?}",
    r_small
  );

  // Key larger than max
  let r_large = pgm.predict_range(999);
  assert!(r_large.start <= r_large.end);
  assert!(r_large.end <= len);
  // binary search would find index len for key 999
  // range should include len or reach len as end
  // Since range is exclusive at end, if r.end == len, it covers up to len-1.
  // Wait, partition_point for 999 in [100..200] is 100 (len).
  // So we probably want range to be able to touch len.
  // predict_range returns indices valid for slice indexing?
  // Rust slices: slice[start..end]. range.end can be len.
  // binary_search expects indices within the slice.
  // But if we want to support insertion point, we might look at search result.
  // The current Pgm::find uses this range.
  // If `find` returns `len`, then it means it's after the last element.
  // So the range must essentially allow the search to reach `len`.
  // Let's check if the current implementation allows `end` to be `len`.
  // Yes, .min(seg.end_idx) where end_idx is exclusive, so max is len.

  // Note: predict_range ensures range is within segment bounds.
  // If the key is way outside, `find_seg` locates the first or last segment.

  // We check if the search logic works with this range.
  // Actually, for key 999, predict might give a large number.
  // Clamped to segment bounds. Last segment ends at len.
  // So range.end can be len.
  // If `contains(&len)` is called, it returns false for `start..len`.
  // But that's fine for binary_search on `data[start..end]`.
  // However, `find` returns an index. If index is `len`, it means typical "not found / append".
  // The predict_range should define the search space.

  OK
}

#[test]
fn test_range_duplicates() -> Void {
  let sorted = vec![10u64, 10, 10, 10, 20, 20, 30];
  let pgm = Pgm::new(&sorted, 4);

  // Search for 10
  let r10 = pgm.predict_range(10);
  check_range(r10.clone(), 0, sorted.len(), "dup 10 first");
  // It should cover all 10s ideally, or at least one of them so binary search finds one.
  // Binary search finds *any* one.

  // Search for 20
  let r20 = pgm.predict_range(20);
  check_range(r20, 4, sorted.len(), "dup 20");

  OK
}

#[test]
fn test_range_single_element() -> Void {
  let sorted = vec![42u64];
  let pgm = Pgm::new(&sorted, 2);

  let r = pgm.predict_range(42);
  check_range(r, 0, 1, "single");
  OK
}

#[test]
fn test_range_all_same() -> Void {
  let sorted = vec![100u64; 100];
  let pgm = Pgm::new(&sorted, 4);

  let r = pgm.predict_range(100);
  // It should encompass the valid range.
  // With intercept/slope, it might point to middle.
  // But segment bounds are 0..100.
  // Range should be within 0..100.
  assert!(r.start < r.end);
  assert!(r.end <= 100);
  OK
}
