#![cfg(feature = "data")]
use jdb_pgm::PgmData;
use rand::{Rng, SeedableRng, rngs::StdRng};

#[test]
fn test_pgm_correctness_random_large() {
  let mut rng = StdRng::seed_from_u64(42);

  const SIZES: [usize; 6] = [0, 1, 10, 128, 1000, 10_000];
  // Test various sizes
  for size in SIZES {
    println!("Testing size: {}", size);

    if size == 0 {
      let pgm = PgmData::new(&[], 8);
      assert!(pgm.get(123).is_none());
      continue;
    }

    let mut data = vec![0u64; size];
    let mut current: u64 = 0;

    // Generate sorted data with large gaps
    for val in data.iter_mut() {
      // Use large jumps to reach u64::MAX range
      let max_jump = u64::MAX / (size as u64);
      let jump = rng.random_range(1..=max_jump);
      // Prevent overflow
      current = current.saturating_add(jump);
      *val = current;
    }

    // Use a reasonable epsilon
    let epsilon = 16;
    let pgm = PgmData::new(&data, epsilon);

    // 1. Validate Random Access (get)
    for (i, &key) in data.iter().enumerate() {
      let pos = pgm.get(key);
      assert_eq!(pos, Some(i), "Get failed at index {} for key {}", i, key);
    }

    // 2. Validate Non-existent keys
    for _ in 0..100 {
      let key = rng.random::<u64>();
      // naive binary search to check existence
      let exists = data.binary_search(&key).is_ok();
      let found = pgm.get(key).is_some();
      assert_eq!(
        found, exists,
        "Existence check failed for random key {}",
        key
      );
    }
  }
}
