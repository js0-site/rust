use jdb_xorf::{Bf8, Filter};

#[test]
fn test_construction_success_rate() {
  let key_count = 1_000_000;
  let iterations = 100;
  let mut success_count = 0;

  for i in 0..iterations {
    let keys: Vec<u64> = (0..key_count).map(|k| k ^ ((i as u64) << 32)).collect();
    // The From trait for BinaryFuse will panic if it fails to build after 100 trials
    // (Actually it might try different seeds internally)
    let filter = Bf8::from(&keys);
    if filter.len() > 0 {
      success_count += 1;
    }
  }

  println!("\n=== Construction Success Rate ===");
  println!("Success: {} / {}", success_count, iterations);
  assert_eq!(
    success_count, iterations,
    "Met construction failure with simplified hash!"
  );
}
