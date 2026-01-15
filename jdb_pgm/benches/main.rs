//! Criterion benchmark comparing Pgm-Index vs binary search vs pgm_index crate
//! Criterion 基准测试：Pgm-Index vs 二分查找 vs pgm_index crate

#[cfg(feature = "pk")]
mod bench_binary;
#[cfg(feature = "pk")]
mod bench_btreemap;
#[cfg(feature = "pk")]
mod bench_external_pgm;
#[cfg(feature = "pk")]
mod bench_hashmap;
mod bench_jdb_pgm;

use std::hint::black_box;

#[cfg(feature = "pk")]
use bench_binary::BinarySearch;
#[cfg(feature = "pk")]
use bench_btreemap::BTreeMapIndex;
#[cfg(feature = "pk")]
use bench_external_pgm::ExternalPgm;
#[cfg(feature = "pk")]
use bench_hashmap::HashMapIndex;
use bench_jdb_pgm::JdbPgm;
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use jdb_pgm::bench_common::Benchmarkable;
use rand::{Rng, SeedableRng, rngs::StdRng};

#[global_allocator]
static ALLOC: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

const SAMPLE_SIZE: usize = 20;
const DATA_SIZES: &[usize] = &[10_000, 100_000, 1_000_000];
const EPSILONS: &[usize] = &[32, 64, 128];
const SEED: u64 = 42;

/// Generate sequential data
/// 生成顺序数据
#[inline]
fn gen_seq(size: usize) -> Vec<u64> {
  (0..size as u64).collect()
}

/// Generate random queries
/// 生成随机查询
#[inline]
fn gen_queries(size: usize, count: usize) -> Vec<u64> {
  let mut rng = StdRng::seed_from_u64(SEED);
  (0..count)
    .map(|_| rng.random_range(0..size as u64))
    .collect()
}

/// Benchmark single lookups for a given implementation
/// 对给定实现进行单次查找基准测试
fn bench_impl<T: Benchmarkable>(
  group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>,
  data: &[u64],
  queries: &[u64],
  size: usize,
  eps: Option<usize>,
) {
  let idx = T::build(data, eps);
  group.bench_with_input(
    BenchmarkId::new(T::bench_name(eps), size),
    &(data, queries),
    |b, (data, queries)| {
      b.iter(|| {
        for &q in queries.iter() {
          black_box(idx.query(data, q));
        }
      })
    },
  );
}

fn bench_single(c: &mut Criterion) {
  let mut group = c.benchmark_group("single_lookups");
  group.sample_size(SAMPLE_SIZE);

  for &size in DATA_SIZES {
    let data = gen_seq(size);
    let queries = gen_queries(size, 1000);
    group.throughput(Throughput::Elements(queries.len() as u64));

    #[cfg(feature = "pk")]
    {
      bench_impl::<BinarySearch>(&mut group, &data, &queries, size, None);
      bench_impl::<HashMapIndex>(&mut group, &data, &queries, size, None);
      bench_impl::<BTreeMapIndex>(&mut group, &data, &queries, size, None);
    }

    for &eps in EPSILONS {
      bench_impl::<JdbPgm>(&mut group, &data, &queries, size, Some(eps));
      #[cfg(feature = "pk")]
      bench_impl::<ExternalPgm>(&mut group, &data, &queries, size, Some(eps));
    }
  }
  group.finish();
}

#[cfg(feature = "pk")]
fn bench_batch(c: &mut Criterion) {
  let mut group = c.benchmark_group("batch_lookups");
  group.sample_size(SAMPLE_SIZE);

  let data = gen_seq(1_000_000);

  for batch in [100, 1_000, 10_000] {
    let queries = gen_queries(1_000_000, batch);
    group.throughput(Throughput::Elements(batch as u64));

    let binary = BinarySearch::build(&data, None);
    group.bench_with_input(
      BenchmarkId::new(BinarySearch::NAME, batch),
      &(&data, &queries),
      |b, (data, queries)| {
        b.iter(|| {
          for &q in queries.iter() {
            black_box(binary.query(data, q));
          }
        })
      },
    );

    let pgm = JdbPgm::build(&data, Some(64));
    group.bench_with_input(
      BenchmarkId::new(JdbPgm::bench_name(Some(64)), batch),
      &(&data, &queries),
      |b, (data, queries)| {
        b.iter(|| {
          for &q in queries.iter() {
            black_box(pgm.query(data, q));
          }
        })
      },
    );
  }
  group.finish();
}

fn bench_build(c: &mut Criterion) {
  let mut group = c.benchmark_group("build_time");
  group.sample_size(SAMPLE_SIZE);

  for &size in DATA_SIZES {
    let data = gen_seq(size);
    group.throughput(Throughput::Elements(size as u64));

    for &eps in EPSILONS {
      group.bench_with_input(
        BenchmarkId::new(JdbPgm::bench_name(Some(eps)), size),
        &(&data, eps),
        |b, (data, eps)| b.iter(|| black_box(JdbPgm::build(data, Some(*eps)))),
      );

      #[cfg(feature = "pk")]
      group.bench_with_input(
        BenchmarkId::new(ExternalPgm::bench_name(Some(eps)), size),
        &(&data, eps),
        |b, (data, eps)| b.iter(|| black_box(ExternalPgm::build(data, Some(*eps)))),
      );
    }
  }
  group.finish();
}

#[cfg(feature = "pk")]
fn bench_compare(c: &mut Criterion) {
  let mut group = c.benchmark_group("jdb_vs_external");
  group.sample_size(SAMPLE_SIZE);

  for &size in DATA_SIZES {
    let data = gen_seq(size);
    let queries = gen_queries(size, 1000);
    group.throughput(Throughput::Elements(queries.len() as u64));

    for &eps in EPSILONS {
      bench_impl::<JdbPgm>(&mut group, &data, &queries, size, Some(eps));
      bench_impl::<ExternalPgm>(&mut group, &data, &queries, size, Some(eps));
    }
  }
  group.finish();
}

#[cfg(feature = "pk")]
criterion_group!(
  benches,
  bench_single,
  bench_batch,
  bench_build,
  bench_compare
);

#[cfg(not(feature = "pk"))]
criterion_group!(benches, bench_single, bench_build);

criterion_main!(benches);
