use jdb_xorf::{Bf16, Bf32, Bf8, Filter};
use rand::Rng;

const SAMPLE_SIZE: usize = 1_000_000;

fn test_initialization_gen<F>()
where
  F: Filter<u64> + for<'a> From<&'a [u64]>,
{
  let mut rng = rand::rng();
  let keys: Vec<u64> = (0..SAMPLE_SIZE).map(|_| rng.random()).collect();
  let filter = F::from(&keys[..]);
  for key in keys {
    assert!(filter.has(&key));
  }
}

fn test_bits_per_entry_gen<F>(bits_per_fingerprint: f64, limit: f64)
where
  F: Filter<u64> + for<'a> From<&'a [u64]>,
{
  let mut rng = rand::rng();
  let keys: Vec<u64> = (0..SAMPLE_SIZE).map(|_| rng.random()).collect();
  let filter = F::from(&keys[..]);
  let bpe = (filter.len() as f64) * bits_per_fingerprint / (SAMPLE_SIZE as f64);
  assert!(bpe < limit, "Bits per entry is {}. Limit: {}", bpe, limit);
}

fn test_false_positives_gen<F>(fp_limit: f64)
where
  F: Filter<u64> + for<'a> From<&'a [u64]>,
{
  use rand::SeedableRng;
  let mut rng = rand::rngs::StdRng::seed_from_u64(42);
  let keys: Vec<u64> = (0..SAMPLE_SIZE).map(|_| rng.random()).collect();
  let filter = F::from(&keys[..]);
  let false_positives: usize = (0..SAMPLE_SIZE)
    .map(|_| rng.random())
    .filter(|n| filter.has(n))
    .count();
  let fp_rate: f64 = (false_positives * 100) as f64 / SAMPLE_SIZE as f64;
  assert!(
    fp_rate < fp_limit,
    "False positive rate is {fp_rate}. Limit: {fp_limit}"
  );
}

#[test]
fn test_bfuse8_initialization() {
  test_initialization_gen::<Bf8>();
}
#[test]
fn test_bfuse16_initialization() {
  test_initialization_gen::<Bf16>();
}
#[test]
fn test_bfuse32_initialization() {
  test_initialization_gen::<Bf32>();
}

#[test]
fn test_bfuse8_bits_per_entry() {
  test_bits_per_entry_gen::<Bf8>(8.0, 9.1);
}
#[test]
fn test_bfuse16_bits_per_entry() {
  test_bits_per_entry_gen::<Bf16>(16.0, 18.1);
}
#[test]
fn test_bfuse32_bits_per_entry() {
  test_bits_per_entry_gen::<Bf32>(32.0, 36.2);
}

#[test]
fn test_bfuse8_false_positives() {
  test_false_positives_gen::<Bf8>(0.406);
}
#[test]
fn test_bfuse16_false_positives() {
  test_false_positives_gen::<Bf16>(0.0025);
}
#[test]
fn test_bfuse32_false_positives() {
  test_false_positives_gen::<Bf32>(0.0000000000000001);
}

