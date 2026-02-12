use jdb_pgm::Pc;
use rand::prelude::*;

#[test]
fn test_pc_correctness_random() {
  let mut rng = StdRng::seed_from_u64(42);

  // Test various sizes
  for size in [0, 1, 10, 128, 129, 256, 1000, 10_000] {
    println!("Testing size: {}", size);
    let mut data = vec![0u64; size];

    // Generate sorted data for better compression behavior,
    // though Pc works with any data (just segments will be short if random).
    // Let's use sorted data to mimic real usage.
    let mut current = 0;
    for val in data.iter_mut() {
      current += rng.random_range(1..u64::MAX as usize / size) as u64;
      *val = current;
    }

    let pc = Pc::new(&data, 8);

    // 1. Validate Consistency
    assert_eq!(pc.len, size);

    // 2. Validate Random Access (get)
    for (i, &val) in data.iter().enumerate() {
      assert_eq!(pc.get(i), Some(val), "Get failed at index {}", i);
    }
    assert_eq!(pc.get(size), None);

    // 3. Validate Forward Iteration
    let collected: Vec<u64> = pc.iter().collect();
    assert_eq!(collected, data, "Forward iter mismatch");

    // 4. Validate Reverse Iteration
    let collected_rev: Vec<u64> = pc.rev_iter().collect();
    let mut data_rev = data.clone();
    data_rev.reverse();
    assert_eq!(collected_rev, data_rev, "Reverse iter mismatch");

    // 5. Validate Serialization loop
    let encoded = pc.dump();
    let loaded = Pc::load(&encoded).unwrap();

    assert_eq!(loaded.len, size);
    // Check fields
    // Segments removed in Block-Local Optimization
    // assert_eq!(loaded.segments.len(), pc.segments.len());
    assert_eq!(loaded.block_meta.len(), pc.block_meta.len());

    for (i, &val) in data.iter().enumerate() {
      assert_eq!(loaded.get(i), Some(val), "Loaded Get failed at index {}", i);
    }
  }
}
