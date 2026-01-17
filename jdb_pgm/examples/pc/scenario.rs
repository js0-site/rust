use rand::prelude::*;
use rand_distr::Distribution;

pub trait Scenario {
  const NAME_ZH: &'static str;
  const NAME_EN: &'static str;
  fn generate(n: usize) -> Vec<u64>;
}

pub struct KeyOffsets;
impl Scenario for KeyOffsets {
  const NAME_ZH: &'static str = "键偏移量 (Zipf)";
  const NAME_EN: &'static str = "Key Offsets (Zipf)";

  fn generate(n: usize) -> Vec<u64> {
    let mut rng = StdRng::seed_from_u64(42);
    let zipf = rand_distr::Zipf::new(100.0, 1.5).unwrap();
    let mut cur = 0u64;
    (0..n)
      .map(|_| {
        cur += zipf.sample(&mut rng) as u64 + 16;
        cur
      })
      .collect()
  }
}

pub struct DocIds;
impl Scenario for DocIds {
  const NAME_ZH: &'static str = "文档ID (Uniform)";
  const NAME_EN: &'static str = "Doc IDs (Uniform)";

  fn generate(n: usize) -> Vec<u64> {
    let mut rng = StdRng::seed_from_u64(42);
    let mut cur = 0u64;
    (0..n)
      .map(|_| {
        cur += rng.random_range(1..100);
        cur
      })
      .collect()
  }
}
