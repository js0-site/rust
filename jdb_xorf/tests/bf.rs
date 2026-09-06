use core::hash::{Hash, Hasher};

use jdb_xorf::{Bf, Bf8, Bf16, Bf32, DefaultHasher, Filter};
use rand::RngExt;

#[test]
fn test_initialization_from() {
  const SAMPLE_SIZE: usize = 1_000_000;
  let keys: Vec<u64> = (0..SAMPLE_SIZE).map(|_| rand::rng().random()).collect();

  fn drive_test<F>(keys: &[u64])
  where
    F: Filter<u64> + for<'a> From<&'a [u64]>,
  {
    let filter = F::from(keys);
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
      let mut hasher = DefaultHasher::default();
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

#[test]
fn test_bf_u64_has_key_and_batch() {
  let keys = vec![111u64, 222, 333, 444, 555];
  let filter: Bf<u64, Bf8> = Bf::from_slice(&keys);

  // Both has and has_key must match and return true for inserted keys
  for &k in &keys {
    assert!(filter.has(&k), "has(&{k}) should be true");
    assert!(filter.has_key(k), "has_key({k}) should be true");
  }

  assert!(!filter.has_key(999999));
  assert!(!filter.has(&999999u64));

  let test_keys = vec![111u64, 999, 333, 888, 555];
  let mut results = vec![false; test_keys.len()];
  filter.contains_batch_keys(&test_keys, &mut results);
  assert_eq!(results, vec![true, false, true, false, true]);
}

#[test]
fn test_new_api_features() {
  let fruits = [
    "apple".to_string(),
    "banana".to_string(),
    "cherry".to_string(),
  ];

  // Test FromIterator / from_iterator / From<Vec<T>>
  let filter: Bf<String, Bf8> = fruits.iter().cloned().collect();
  let filter2: Bf<String, Bf8> = Bf::from_iterator(fruits.iter().cloned());
  let filter3: Bf<String, Bf8> = Bf::from_vec(fruits.to_vec());
  let filter4: Bf<String, Bf8> = fruits.to_vec().into();
  assert!(!filter.is_empty());
  assert!(filter.len() >= 3);
  assert!(filter.bytes() > 0);
  assert_eq!(filter.bits(), filter.bytes() * 8);
  assert!(filter2.has("banana"));
  assert!(filter3.has("cherry"));
  assert!(filter4.has("apple"));

  // Test has_hash
  let mut hasher = DefaultHasher::default();
  "apple".hash(&mut hasher);
  let apple_hash = hasher.finish();
  assert!(filter.has_hash(apple_hash));
  assert!(!filter.has_hash(0xdead_beef_1234_5678));

  // Test contains_batch and contains_batch_vec on Bf
  let test_items = ["apple", "mango", "banana", "watermelon"];
  let res = filter.contains_batch_vec(&test_items);
  assert_eq!(res, vec![true, false, true, false]);

  let mut buf = [false; 4];
  filter.contains_batch(&test_items, &mut buf);
  assert_eq!(buf, [true, false, true, false]);

  // Test Base::contains_batch_vec
  let u64_keys = vec![111u64, 222, 333];
  let base_filter = Bf8::from(&u64_keys);
  assert!(base_filter.bytes() > 0);
  assert_eq!(base_filter.bits(), base_filter.bytes() * 8);
  let batch_u64 = base_filter.contains_batch_vec(&[111, 999, 222]);
  assert_eq!(batch_u64, vec![true, false, true]);

  // Test Bf::from_hashes
  let hashes = vec![apple_hash, 123456789, 987654321];
  let hash_filter: Bf<String, Bf8> = Bf::from_hashes(hashes);
  assert!(hash_filter.has_hash(apple_hash));
  assert!(hash_filter.has_hash(123456789));
  assert!(!hash_filter.has_hash(555555));
}

#[cfg(feature = "museair")]
#[test]
fn test_museair_hasher() {
  use jdb_xorf::MuseairHasher;

  let fruits = vec![
    "apple".to_string(),
    "banana".to_string(),
    "pear".to_string(),
  ];
  let filter: Bf<String, Bf8, MuseairHasher> = Bf::from(&fruits);

  assert!(filter.has("apple"));
  assert!(filter.has("banana"));
  assert!(filter.has("pear"));
  assert!(!filter.has("watermelon"));
}

#[test]
fn test_hasher_perf() {
  use std::time::Instant;

  const N: usize = 100_000;
  let strings: Vec<String> = (0..N).map(|i| format!("key_{i:016x}")).collect();

  // DefaultHasher direct
  let start = Instant::now();
  let mut sum_gx = 0u64;
  for s in &strings {
    let mut h = DefaultHasher::default();
    s.hash(&mut h);
    sum_gx = sum_gx.wrapping_add(h.finish());
  }
  let dur_gx = start.elapsed();
  let mops_gx = (N as f64) / dur_gx.as_secs_f64() / 1_000_000.0;
  println!("\n=== Hasher Direct Hashing (100k 20B keys) ===");
  println!("DefaultHasher:  {dur_gx:?} ({mops_gx:.2} M ops/s) [sum={sum_gx}]");

  // Std DefaultHasher
  use std::collections::hash_map::DefaultHasher as StdHasher;
  let start = Instant::now();
  let mut sum_std = 0u64;
  for s in &strings {
    let mut h = StdHasher::new();
    s.hash(&mut h);
    sum_std = sum_std.wrapping_add(h.finish());
  }
  let dur_std = start.elapsed();
  let mops_std = (N as f64) / dur_std.as_secs_f64() / 1_000_000.0;
  println!("StdHasher:      {dur_std:?} ({mops_std:.2} M ops/s) [sum={sum_std}]");

  #[cfg(feature = "museair")]
  {
    use jdb_xorf::MuseairHasher;
    let start = Instant::now();
    let mut sum_mu = 0u64;
    for s in &strings {
      let mut h = MuseairHasher::default();
      s.hash(&mut h);
      sum_mu = sum_mu.wrapping_add(h.finish());
    }
    let dur_mu = start.elapsed();
    let mops_mu = (N as f64) / dur_mu.as_secs_f64() / 1_000_000.0;
    println!("MuseairHasher:  {dur_mu:?} ({mops_mu:.2} M ops/s) [sum={sum_mu}]");
  }

  println!("\n=== Bf<String, Bf8> Build & Query (100k keys) ===");
  let start = Instant::now();
  let filter_gx: Bf<String, Bf8> = Bf::from(&strings);
  let build_gx = start.elapsed();
  let build_mops_gx = (N as f64) / build_gx.as_secs_f64() / 1_000_000.0;

  let start = Instant::now();
  let mut found = 0;
  for s in &strings {
    if filter_gx.has(s) {
      found += 1;
    }
  }
  let query_gx = start.elapsed();
  let query_mops_gx = (N as f64) / query_gx.as_secs_f64() / 1_000_000.0;
  println!("Bf<String, Bf8, DefaultHasher>:");
  println!("  Build: {build_gx:?} ({build_mops_gx:.2} M ops/s)");
  println!("  Query: {query_gx:?} ({query_mops_gx:.2} M ops/s), found={found}");
}

#[test]
fn test_zero_keys_and_default() {
  // Empty Base / Bf8
  let empty_base = Bf8::default();
  assert_eq!(empty_base.len(), 0);
  assert!(empty_base.is_empty());
  assert!(!empty_base.has_key(123));
  assert!(!empty_base.has(&123u64));

  let from_empty = Bf8::from(&[][..]);
  assert_eq!(from_empty.len(), 0);
  assert!(from_empty.is_empty());
  assert!(!from_empty.has_key(456));

  let try_empty = Bf8::try_from_slice([]).unwrap();
  assert_eq!(try_empty.len(), 0);
  assert!(try_empty.is_empty());

  // Empty Bf
  let empty_bf: Bf<String, Bf8> = Bf::default();
  assert_eq!(empty_bf.len(), 0);
  assert!(empty_bf.is_empty());
  assert!(!empty_bf.has("hello"));

  // Empty batch query
  let mut results = [];
  empty_base.contains_batch(&[], &mut results);
  assert_eq!(empty_base.contains_batch_vec(&[]), Vec::<bool>::new());
}

#[test]
fn test_single_key() {
  let keys = [42u64];
  let filter = Bf8::from(&keys[..]);
  assert!(!filter.is_empty());
  assert!(filter.has(&42u64));
  assert!(filter.has_key(42));
  assert!(!filter.has(&43u64));
  assert!(!filter.has_key(43));

  let single_str = ["hello".to_string()];
  let bf_str: Bf<String, Bf8> = single_str.to_vec().into();
  assert!(bf_str.has("hello"));
  assert!(!bf_str.has("world"));
}

#[test]
fn test_bf64() {
  use jdb_xorf::Bf64;

  let keys: Vec<u64> = vec![111111111, 222222222, 333333333];
  let filter = Bf64::from(&keys[..]);
  assert!(!filter.is_empty());
  for &k in &keys {
    assert!(filter.has_key(k));
    assert!(filter.has(&k));
  }
  assert!(!filter.has_key(999999999));

  let res = filter.contains_batch_vec(&[111111111, 999999999, 222222222]);
  assert_eq!(res, vec![true, false, true]);
}

#[test]
fn test_cast_and_unsized_conversions() {
  let words = vec!["alpha".to_string(), "beta".to_string(), "gamma".to_string()];
  let filter: Bf<String, Bf8> = Bf::from(&words);

  // Test clone and debug
  let cloned = filter.clone();
  assert_eq!(filter, cloned);
  let _ = format!("{filter:?}");

  // Test cast to unsized str
  let str_filter: Bf<str, Bf8> = filter.cast();
  assert!(str_filter.has("alpha"));
  assert!(str_filter.has("beta"));
  assert!(!str_filter.has("omega"));

  // Test From<Bf<String>> for Bf<str>
  let filter2: Bf<String, Bf8> = Bf::from(&words);
  let str_filter2: Bf<str, Bf8> = filter2.into();
  assert!(str_filter2.has("gamma"));

  // Test From<Bf<Vec<u8>>> for Bf<[u8]>
  let byte_keys = vec![vec![1, 2, 3], vec![4, 5, 6]];
  let byte_filter: Bf<Vec<u8>, Bf8> = Bf::from(&byte_keys);
  let slice_filter: Bf<[u8], Bf8> = byte_filter.into();
  assert!(slice_filter.has([1u8, 2, 3].as_slice()));
  assert!(!slice_filter.has([9u8, 9, 9].as_slice()));

  // Test Bf<str>::from_hashes
  let mut hasher = DefaultHasher::default();
  "test_str".hash(&mut hasher);
  let h = hasher.finish();
  let from_h: Bf<str, Bf8> = Bf::from_hashes(vec![h, 777]);
  assert!(from_h.has_hash(h));
}

#[test]
fn test_batch_various_lengths() {
  let keys: Vec<u64> = (1..=20).collect();
  let filter = Bf8::from(&keys[..]);

  // Test slice lengths: 0, 1, 2, 3, 4, 5, 7, 8, 9, 15, 16, 17
  for len in [0, 1, 2, 3, 4, 5, 7, 8, 9, 15, 16, 17] {
    let test_keys: Vec<u64> = (1..=len as u64).collect();
    let res = filter.contains_batch_vec(&test_keys);
    assert_eq!(res.len(), len);
    assert!(res.iter().all(|&x| x));

    let mut buf = vec![false; len];
    filter.contains_batch(&test_keys, &mut buf);
    assert!(buf.iter().all(|&x| x));
  }
}

#[test]
fn test_from_iterator_for_base() {
  let base: Bf8 = (100u64..200u64).collect();
  assert!(!base.is_empty());
  assert!(base.has_key(150));
  assert!(!base.has_key(250));
}

#[test]
fn test_try_from_iterator_and_try_from_hashes() {
  let items = vec!["one".to_string(), "two".to_string()];
  let filter: Bf<String, Bf8> = Bf::try_from_iterator(items.clone()).expect("should build");
  assert!(filter.has("one"));

  let filter_vec: Bf<String, Bf8> = Bf::try_from_vec(items).expect("should build");
  assert!(filter_vec.has("two"));

  let hashes = vec![11111u64, 22222, 33333];
  let h_filter: Bf<String, Bf8> = Bf::try_from_hashes(hashes).expect("should build");
  assert!(h_filter.has_hash(11111));
  assert!(!h_filter.has_hash(99999));
}

#[test]
fn test_mix_hasher() {
  use jdb_xorf::MixHasher;

  let items = ["alpha".to_string(), "beta".to_string()];
  let filter: Bf<String, Bf8, MixHasher> = Bf::from(&items);
  assert!(filter.has("alpha"));
  assert!(filter.has("beta"));
  assert!(!filter.has("gamma"));

  // Verify short byte slices with matching length/value do not collide
  let mut h1 = MixHasher::default();
  h1.write(&[1u8]);
  let mut h2 = MixHasher::default();
  h2.write(&[2u8, 0u8]);
  let mut h3 = MixHasher::default();
  h3.write(&[3u8, 0u8, 0u8]);
  assert_ne!(h1.finish(), h2.finish());
  assert_ne!(h2.finish(), h3.finish());
  assert_ne!(h1.finish(), h3.finish());

  // Verify write_u8 matches write(&[u8])
  let mut h_u8 = MixHasher::default();
  h_u8.write_u8(42);
  let mut h_slice = MixHasher::default();
  h_slice.write(&[42u8]);
  assert_eq!(h_u8.finish(), h_slice.finish());
}

#[test]
fn test_is_valid() {
  let keys = [10u64, 20, 30];
  let filter = Bf8::from(&keys[..]);
  assert!(filter.is_valid());

  let empty = Bf8::default();
  assert!(empty.is_valid());

  // Corrupted descriptor
  let mut corrupted = filter.clone();
  corrupted.desc.seg_len = 3; // not power of two
  assert!(!corrupted.is_valid());
}

#[test]
fn test_batch_with_strings() {
  let keys = vec!["apple".to_string(), "banana".to_string(), "cherry".to_string()];
  let filter: Bf<String, Bf8> = Bf::from(&keys);

  // Test borrowed &[&str]
  let query_refs = ["apple", "unknown", "banana"];
  let res = filter.contains_batch_vec(&query_refs);
  assert_eq!(res, vec![true, false, true]);

  let mut buf = [false; 3];
  filter.contains_batch(&query_refs, &mut buf);
  assert_eq!(buf, [true, false, true]);

  // Test owned &[String] via contains_batch_items
  let query_strings = vec!["apple".to_string(), "unknown".to_string(), "banana".to_string()];
  let res_owned = filter.contains_batch_items_vec(&query_strings);
  assert_eq!(res_owned, vec![true, false, true]);

  let mut buf_owned = [false; 3];
  filter.contains_batch_items(&query_strings, &mut buf_owned);
  assert_eq!(buf_owned, [true, false, true]);
}
