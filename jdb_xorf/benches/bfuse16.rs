#[macro_use]
extern crate criterion;
extern crate core;
extern crate jdb_xorf;
extern crate rand;

use core::convert::TryFrom;

use criterion::{BenchmarkId, Criterion};
use jdb_xorf::{BinaryFuse16, Filter};
use rand::Rng;

const SAMPLE_SIZE: u32 = 500_000;

fn from(c: &mut Criterion) {
  let mut group = c.benchmark_group("BinaryFuse16");
  let group = group.sample_size(10);

  let mut rng = rand::rng();
  let keys: Vec<u64> = (0..SAMPLE_SIZE).map(|_| rng.random()).collect();

  group.bench_with_input(BenchmarkId::new("from", SAMPLE_SIZE), &keys, |b, keys| {
    b.iter(|| BinaryFuse16::try_from(keys).unwrap());
  });
}

fn contains(c: &mut Criterion) {
  let mut group = c.benchmark_group("BinaryFuse16");

  let mut rng = rand::rng();
  let keys: Vec<u64> = (0..SAMPLE_SIZE).map(|_| rng.random()).collect();
  let filter = BinaryFuse16::try_from(&keys).unwrap();

  group.bench_function(BenchmarkId::new("contains", SAMPLE_SIZE), |b| {
    let key = rng.random();
    b.iter(|| filter.contains(&key));
  });
}

criterion_group!(bfuse16, from, contains);
criterion_main!(bfuse16);
