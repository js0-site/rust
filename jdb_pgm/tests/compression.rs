use jdb_pgm::Pc;

#[test]
fn test_pc_workflow() {
  let data: Vec<u64> = (0..1000).collect();
  let epsilon = 16;

  // 1. Build using Pc
  let pc = Pc::new(&data, epsilon);

  // 2. Verify basic properties
  assert_eq!(pc.len, 1000);
  assert!(!pc.segments.is_empty());
  assert!(!pc.block_meta.is_empty());

  // 3. Verify Random Access
  for i in 0..1000 {
    assert_eq!(pc.get(i), Some(i as u64), "Failed at index {}", i);
  }
  assert_eq!(pc.get(1000), None);

  // 4. Verify Iteration
  let mut count = 0;
  for (i, val) in pc.iter().enumerate() {
    assert_eq!(val, i as u64);
    count += 1;
  }
  assert_eq!(count, 1000);

  // 5. Verify Reverse Iteration
  let mut count = 0;
  for (i, val) in pc.rev_iter().enumerate() {
    assert_eq!(val, (999 - i) as u64);
    count += 1;
  }
  assert_eq!(count, 1000);

  // 6. Verify Serialization
  let encoded = pc.dump();
  let loaded = Pc::load(&encoded);
  assert_eq!(loaded.len, pc.len);
  for i in 0..1000 {
    assert_eq!(loaded.get(i), pc.get(i));
  }
}

#[test]
fn test_empty_pc() {
  let data: Vec<u64> = vec![];
  let pc = Pc::new(&data, 16);
  assert_eq!(pc.len, 0);
  assert_eq!(pc.segments.len(), 0);
  assert_eq!(pc.block_meta.len(), 0);
  assert_eq!(pc.get(0), None);
  assert_eq!(pc.iter().count(), 0);
}
