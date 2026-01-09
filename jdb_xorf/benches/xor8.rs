#[macro_use]
extern crate criterion;
extern crate jdb_xorf;
extern crate rand;

use criterion::{BenchmarkId, Criterion};
use jdb_xorf::{Filter, Xor8};
use rand::Rng;

const SAMPLE_SIZE: u32 = 500_000;

fn from(c: &mut Criterion) {
  let mut group = c.benchmark_group("Xor8");
  let group = group.sample_size(10);

  let mut rng = rand::rng();
  let keys: Vec<u64> = (0..SAMPLE_SIZE).map(|_| rng.random()).collect();

  group.bench_with_input(BenchmarkId::new("from", SAMPLE_SIZE), &keys, |b, keys| {
    b.iter(|| Xor8::from(keys));
  });
}

fn contains(c: &mut Criterion) {
  let mut group = c.benchmark_group("Xor8");

  let mut rng = rand::rng();
  let keys: Vec<u64> = (0..SAMPLE_SIZE).map(|_| rng.random()).collect();
  let filter = Xor8::from(&keys);

  group.bench_function(BenchmarkId::new("contains", SAMPLE_SIZE), |b| {
    let key = rng.random();
    b.iter(|| filter.contains(&key));
  });
}

criterion_group!(xor8, from, contains);
criterion_main!(xor8);
