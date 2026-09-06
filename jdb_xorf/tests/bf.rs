use core::hash::{Hash, Hasher};

use jdb_xorf::{Bf, Bf8, Bf16, Bf32, Filter, RapidHasher};
use rand::RngExt;

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
      assert!(filter.has(key));
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
      let mut hasher = RapidHasher::default();
      k.hash(&mut hasher);
      hasher.finish()
    })
    .collect();

  let filter = Bf8::from(&hashed_keys);

  assert!(filter.has(&hashed_keys[0]));
  assert!(filter.has(&hashed_keys[1]));
  assert!(filter.has(&hashed_keys[2]));
}

#[test]
fn test_duplicate_keys() {
  let keys = vec!["apple", "banana", "apple", "cherry", "banana"];
  let filter = Bf::<&str, Bf8>::from(&keys);

  assert!(filter.has("apple"));
  assert!(filter.has("banana"));
  assert!(filter.has("cherry"));
}

#[test]
fn test_try_from_and_batch() {
  let keys = vec![100u64, 200, 300, 400, 500];
  let filter = Bf8::try_from_slice(&keys).expect("construction should succeed");

  // has_key direct query
  for &k in &keys {
    assert!(filter.has_key(k));
  }
  assert!(!filter.has_key(999999));

  // contains_batch
  let test_keys = vec![100u64, 999, 300, 888, 500];
  let mut results = vec![false; test_keys.len()];
  filter.contains_batch(&test_keys, &mut results);

  assert_eq!(results, vec![true, false, true, false, true]);
}

#[test]
fn test_typed_try_from_keys() {
  let fruits = vec!["orange".to_string(), "peach".to_string()];
  let filter: Bf<String, Bf8> = Bf::try_from_keys(&fruits).expect("try_from_keys should succeed");

  assert!(filter.has("orange"));
  assert!(filter.has("peach"));
  assert!(!filter.has("grape"));
}
