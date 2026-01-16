use core::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

use jdb_xorf::{Bf, Bf8, Bf16, Bf32, Filter};
use rand::Rng;

#[test]
fn test_initialization_from() {
  const SAMPLE_SIZE: usize = 1_000_000;
  let keys: Vec<u64> = (0..SAMPLE_SIZE).map(|_| rand::rng().random()).collect();

  fn drive_test<F>(keys: &[u64])
  where
    F: Filter<u64> + From<Vec<u64>>,
  {
    let filter = F::from(keys.to_vec());
    for key in keys {
      assert!(filter.contains(key));
    }
  }

  drive_test::<Bf8>(&keys);
  drive_test::<Bf16>(&keys);
  drive_test::<Bf32>(&keys);
}

#[test]
fn test_borrow_query() {
  let keys: Vec<String> = vec![
    "apple".to_string(),
    "banana".to_string(),
    "orange".to_string(),
  ];

  let hashed_keys: Vec<u64> = keys
    .iter()
    .map(|k: &String| {
      let mut hasher = DefaultHasher::default();
      k.hash(&mut hasher);
      hasher.finish()
    })
    .collect();

  let filter = Bf8::from(&hashed_keys);

  assert!(filter.contains(&hashed_keys[0]));
  assert!(filter.contains(&hashed_keys[1]));
  assert!(filter.contains(&hashed_keys[2]));
}

#[test]
fn test_duplicate_keys() {
  let keys = vec!["apple", "banana", "apple", "cherry", "banana"];
  let filter = Bf::<&str, Bf8>::from(&keys);

  assert!(filter.contains("apple"));
  assert!(filter.contains("banana"));
  assert!(filter.contains("cherry"));
}
