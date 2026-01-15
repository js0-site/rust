#[macro_use]
extern crate criterion;
extern crate core;
extern crate jdb_xorf;
extern crate rand;

use core::convert::TryFrom;

use criterion::{BenchmarkId, Criterion};
use jdb_xorf::{BinaryFuse8, Filter};
use rand::Rng;

const SAMPLE_SIZE: u32 = 500_000;

fn serialization(c: &mut Criterion) {
  let mut group = c.benchmark_group("BinaryFuse8");
  let group = group.sample_size(10);

  let mut rng = rand::rng();
  let keys: Vec<u64> = (0..SAMPLE_SIZE).map(|_| rng.random()).collect();

  let filter = BinaryFuse8::try_from(keys).unwrap();

  group.bench_with_input(
    BenchmarkId::new("bitcode-serialize", SAMPLE_SIZE),
    &filter,
    |b, filter| {
      b.iter(|| bitcode::encode(filter));
    },
  );

  let serialized_filter = bitcode::encode(&filter);

  group.bench_with_input(
    BenchmarkId::new("bitcode-deserialize", SAMPLE_SIZE),
    &serialized_filter,
    |b, filter| {
      b.iter(|| bitcode::decode::<BinaryFuse8>(filter));
    },
  );
}

fn from(c: &mut Criterion) {
  let mut group = c.benchmark_group("BinaryFuse8");
  let group = group.sample_size(10);

  let mut rng = rand::rng();
  let keys: Vec<u64> = (0..SAMPLE_SIZE).map(|_| rng.random()).collect();

  group.bench_with_input(BenchmarkId::new("from", SAMPLE_SIZE), &keys, |b, keys| {
    b.iter(|| BinaryFuse8::try_from(keys).unwrap());
  });
}

fn contains(c: &mut Criterion) {
  let mut group = c.benchmark_group("BinaryFuse8");

  let mut rng = rand::rng();
  let keys: Vec<u64> = (0..SAMPLE_SIZE).map(|_| rng.random()).collect();
  let filter = BinaryFuse8::try_from(&keys).unwrap();

  group.bench_function(BenchmarkId::new("contains", SAMPLE_SIZE), |b| {
    let key = rng.random();
    b.iter(|| filter.contains(&key));
  });
}

criterion_group!(bfuse8, serialization, from, contains);
criterion_main!(bfuse8);
