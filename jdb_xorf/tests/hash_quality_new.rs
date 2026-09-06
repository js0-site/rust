use jdb_xorf::mix64;

#[test]
fn test_hash_quality_metrics() {
  let mut total_flipped_bits = 0;
  let iterations = 100_000;
  let mut rng = 0x1234567812345678u64;

  fn simple_rng(seed: &mut u64) -> u64 {
    *seed = seed
      .wrapping_mul(6364136223846793005)
      .wrapping_add(1442695040888963407);
    *seed
  }

  for _ in 0..iterations {
    let input = simple_rng(&mut rng);
    let h1 = mix64(input);

    let bit_to_flip = (simple_rng(&mut rng) % 64) as u32;
    let input2 = input ^ (1 << bit_to_flip);
    let h2 = mix64(input2);

    let diff = h1 ^ h2;
    total_flipped_bits += diff.count_ones();
  }

  let avg_flipped = total_flipped_bits as f64 / iterations as f64;
  let percentage = (avg_flipped / 64.0) * 100.0;

  println!("\n=== Hash Quality (Multiply Only) ===");
  println!("Average bits flipped: {avg_flipped:.2} / 64 ({percentage:.2}%)");
}
