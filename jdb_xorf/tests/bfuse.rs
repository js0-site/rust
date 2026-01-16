use jdb_xorf::{
  Bf16, Bf16Ref, Bf32, Bf8, Bf8Ref, Bf32Ref,
  DmaSerializable, Filter, FilterRef,
};
use rand::Rng;

const SAMPLE_SIZE: usize = 1_000_000;

fn test_initialization_gen<F>()
where
  F: Filter<u64> + for<'a> TryFrom<&'a [u64]>,
  for<'a> <F as TryFrom<&'a [u64]>>::Error: core::fmt::Debug,
{
  let mut rng = rand::rng();
  let keys: Vec<u64> = (0..SAMPLE_SIZE).map(|_| rng.random()).collect();
  let filter = F::try_from(&keys[..]).unwrap();
  for key in keys {
    assert!(filter.contains(&key));
  }
}

fn test_bits_per_entry_gen<F>(bits_per_fingerprint: f64, limit: f64)
where
  F: Filter<u64> + for<'a> TryFrom<&'a [u64]>,
  for<'a> <F as TryFrom<&'a [u64]>>::Error: core::fmt::Debug,
{
  let mut rng = rand::rng();
  let keys: Vec<u64> = (0..SAMPLE_SIZE).map(|_| rng.random()).collect();
  let filter = F::try_from(&keys[..]).unwrap();
  let bpe = (filter.len() as f64) * bits_per_fingerprint / (SAMPLE_SIZE as f64);
  assert!(bpe < limit, "Bits per entry is {}. Limit: {}", bpe, limit);
}

fn test_false_positives_gen<F>(fp_limit: f64)
where
  F: Filter<u64> + for<'a> TryFrom<&'a [u64]>,
  for<'a> <F as TryFrom<&'a [u64]>>::Error: core::fmt::Debug,
{
  use rand::SeedableRng;
  let mut rng = rand::rngs::StdRng::seed_from_u64(42);
  let keys: Vec<u64> = (0..SAMPLE_SIZE).map(|_| rng.random()).collect();
  let filter = F::try_from(&keys[..]).unwrap();
  let false_positives: usize = (0..SAMPLE_SIZE)
    .map(|_| rng.random())
    .filter(|n| filter.contains(n))
    .count();
  let fp_rate: f64 = (false_positives * 100) as f64 / SAMPLE_SIZE as f64;
  assert!(
    fp_rate < fp_limit,
    "False positive rate is {fp_rate}. Limit: {fp_limit}"
  );
}

macro_rules! test_dma_roundtrip {
  ($filter_ty:ty, $ref_ty:ty) => {
    let mut rng = rand::rng();
    let keys: Vec<u64> = (0..SAMPLE_SIZE).map(|_| rng.random()).collect();
    let filter = <$filter_ty>::try_from(&keys[..]).unwrap();

    let mut desc = vec![0u8; <$filter_ty>::LEN];
    filter.dma_copy_desc_to(&mut desc);
    let filter_ref = <$ref_ty>::from_dma(&desc, filter.dma_fingerprints());

    for key in &keys {
      assert!(filter_ref.contains(key));
    }
  };
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

#[test]
#[cfg(debug_assertions)]
#[should_panic]
fn test_bfuse8_duplicates() {
  let _ = Bf8::from(vec![1, 2, 1]);
}
#[test]
#[cfg(debug_assertions)]
#[should_panic]
fn test_bfuse16_duplicates() {
  let _ = Bf16::from(vec![1, 2, 1]);
}
#[test]
#[cfg(debug_assertions)]
#[should_panic]
fn test_bfuse32_duplicates() {
  let _ = Bf32::from(vec![1, 2, 1]);
}

#[test]
fn test_bfuse8_dma_roundtrip() {
  test_dma_roundtrip!(Bf8, Bf8Ref);
}
#[test]
fn test_bfuse16_dma_roundtrip() {
  test_dma_roundtrip!(Bf16, Bf16Ref);
}
#[test]
fn test_bfuse32_dma_roundtrip() {
  test_dma_roundtrip!(Bf32, Bf32Ref);
}
