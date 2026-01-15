//! Performance comparison benchmark, outputs JSON.
//! 性能对比基准测试，输出 JSON

use std::{hint::black_box, io::Write, time::Instant};

use autoscale_cuckoo_filter::CuckooFilterBuilder;
use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group};
use cuckoofilter::CuckooFilter as OriginalCuckooFilter;
use farmhash::FarmHasher;
use gxhash::GxHasher;
use mimalloc::MiMalloc;
use scalable_cuckoo_filter::ScalableCuckooFilter;
use serde::Serialize;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

// Criterion benchmark config
// Criterion 基准测试配置
const CAPACITY: usize = 5_000;
const FPP: f64 = 0.01;

// JSON output config
// JSON 输出配置
const ITEMS: usize = 100_000;
const BENCH_CAP: usize = ITEMS * 2;
// cuckoofilter: 8-bit fp, use 10x capacity for similar FPP
// cuckoofilter: 8位指纹，使用10倍容量达到相似 FPP
const CUCKOO_CAP: usize = ITEMS * 10;
const REMOVE_N: usize = 10_000;
const LOOPS: usize = 1000;
const FPP_N: usize = 100_000;

const JSON_PATH: &str = "bench.json";

#[derive(Serialize)]
struct BenchResult {
  lib: &'static str,
  add_mops: f64,
  contains_mops: f64,
  remove_mops: f64,
  memory_kb: f64,
  fpp: f64,
}

#[derive(Serialize)]
struct BenchOutput {
  items: usize,
  capacity: usize,
  target_fpp: f64,
  results: Vec<BenchResult>,
}

fn gen_keys(n: usize, seed: u64) -> Vec<Vec<u8>> {
  fastrand::seed(seed);
  (0..n)
    .map(|_| {
      let len = fastrand::usize(8..=128);
      (0..len).map(|_| fastrand::u8(..)).collect()
    })
    .collect()
}

#[inline]
fn ns_to_mops(ns: f64) -> f64 {
  1000.0 / ns
}

/// Trait for unified filter operations.
/// 统一过滤器操作的 trait
trait Filter {
  fn add_key(&mut self, key: &[u8]);
  fn has_key(&self, key: &[u8]) -> bool;
  fn rm_key(&mut self, key: &[u8]);
  fn mem_bits(&self) -> u64;
}

impl Filter for autoscale_cuckoo_filter::CuckooFilter<[u8], GxHasher> {
  fn add_key(&mut self, key: &[u8]) {
    self.add_if_not_exist(key);
  }
  fn has_key(&self, key: &[u8]) -> bool {
    self.contains(key)
  }
  fn rm_key(&mut self, key: &[u8]) {
    self.remove(key);
  }
  fn mem_bits(&self) -> u64 {
    self.bits()
  }
}

impl Filter for ScalableCuckooFilter<[u8]> {
  fn add_key(&mut self, key: &[u8]) {
    self.insert_if_not_contained(key);
  }
  fn has_key(&self, key: &[u8]) -> bool {
    self.contains(key)
  }
  fn rm_key(&mut self, key: &[u8]) {
    self.remove(key);
  }
  fn mem_bits(&self) -> u64 {
    self.bits()
  }
}

impl Filter for OriginalCuckooFilter<FarmHasher> {
  fn add_key(&mut self, key: &[u8]) {
    let _ = self.add(key);
  }
  fn has_key(&self, key: &[u8]) -> bool {
    self.contains(key)
  }
  fn rm_key(&mut self, key: &[u8]) {
    self.delete(key);
  }
  fn mem_bits(&self) -> u64 {
    (self.memory_usage() * 8) as u64
  }
}

fn bench_filter(
  filter: &mut impl Filter,
  keys: &[Vec<u8>],
  remove_keys: &[&Vec<u8>],
  fpp_keys: &[Vec<u8>],
) -> (f64, f64, f64, f64, f64) {
  // Add
  let start = Instant::now();
  for k in keys {
    filter.add_key(k);
  }
  let add_ns = start.elapsed().as_nanos() as f64 / keys.len() as f64;

  // Contains
  let start = Instant::now();
  for _ in 0..LOOPS {
    for k in keys {
      black_box(filter.has_key(k));
    }
  }
  let contains_ns = start.elapsed().as_nanos() as f64 / (keys.len() * LOOPS) as f64;

  let mem = filter.mem_bits();

  // FPP
  let fp: usize = fpp_keys.iter().filter(|k| filter.has_key(k)).count();
  let fpp = fp as f64 / fpp_keys.len() as f64;

  // Remove
  let start = Instant::now();
  for k in remove_keys {
    filter.rm_key(k);
  }
  let remove_ns = start.elapsed().as_nanos() as f64 / remove_keys.len() as f64;

  (
    ns_to_mops(add_ns),
    ns_to_mops(contains_ns),
    ns_to_mops(remove_ns),
    mem as f64 / 8192.0,
    fpp,
  )
}

pub fn run_bench_json() {
  let keys = gen_keys(ITEMS, 12345);
  let remove_keys: Vec<_> = keys.iter().take(REMOVE_N).collect();
  let fpp_keys = gen_keys(FPP_N, 99999);

  let mut results = Vec::with_capacity(3);

  // autoscale_cuckoo_filter
  {
    let mut f = CuckooFilterBuilder::new()
      .initial_capacity(BENCH_CAP)
      .false_positive_probability(FPP)
      .hasher(GxHasher::default())
      .finish::<[u8]>();
    let (add, contains, remove, mem, fpp) = bench_filter(&mut f, &keys, &remove_keys, &fpp_keys);
    results.push(BenchResult {
      lib: "autoscale_cuckoo_filter",
      add_mops: add,
      contains_mops: contains,
      remove_mops: remove,
      memory_kb: mem,
      fpp,
    });
  }

  // cuckoo_filter
  {
    let mut f = ScalableCuckooFilter::<[u8]>::new(BENCH_CAP, FPP);
    let (add, contains, remove, mem, fpp) = bench_filter(&mut f, &keys, &remove_keys, &fpp_keys);
    results.push(BenchResult {
      lib: "cuckoo_filter",
      add_mops: add,
      contains_mops: contains,
      remove_mops: remove,
      memory_kb: mem,
      fpp,
    });
  }

  // cuckoofilter
  {
    let mut f: OriginalCuckooFilter<FarmHasher> = OriginalCuckooFilter::with_capacity(CUCKOO_CAP);
    let (add, contains, remove, mem, fpp) = bench_filter(&mut f, &keys, &remove_keys, &fpp_keys);
    results.push(BenchResult {
      lib: "cuckoofilter",
      add_mops: add,
      contains_mops: contains,
      remove_mops: remove,
      memory_kb: mem,
      fpp,
    });
  }

  let output = BenchOutput {
    items: ITEMS,
    capacity: BENCH_CAP,
    target_fpp: FPP,
    results,
  };

  let json = sonic_rs::to_string_pretty(&output).expect("serialize json");
  std::fs::File::create(JSON_PATH)
    .expect("create file")
    .write_all(json.as_bytes())
    .expect("write file");
  println!("Saved to {JSON_PATH}");
}

fn bench_insert(c: &mut Criterion) {
  let mut g = c.benchmark_group("insert");
  g.sample_size(50);
  g.warm_up_time(std::time::Duration::from_secs(1));
  g.measurement_time(std::time::Duration::from_secs(2));

  g.bench_function("autoscale_cuckoo_filter", |b| {
    let mut f = CuckooFilterBuilder::new()
      .initial_capacity(CAPACITY)
      .false_positive_probability(FPP)
      .hasher(GxHasher::default())
      .finish::<u64>();
    let mut i = 0u64;
    b.iter(|| {
      f.add_if_not_exist(&i);
      i += 1;
    })
  });

  g.bench_function("cuckoo_filter", |b| {
    let mut f = ScalableCuckooFilter::<u64>::new(CAPACITY, FPP);
    let mut i = 0u64;
    b.iter(|| {
      f.insert_if_not_contained(&i);
      i += 1;
    })
  });

  g.bench_function("cuckoofilter", |b| {
    let mut f: OriginalCuckooFilter<FarmHasher> = OriginalCuckooFilter::with_capacity(CAPACITY);
    let mut i = 0u64;
    b.iter(|| {
      let _ = f.test_and_add(&i);
      i += 1;
    })
  });

  g.finish();
}

fn bench_contains(c: &mut Criterion) {
  let mut g = c.benchmark_group("contains");
  g.sample_size(50);
  g.warm_up_time(std::time::Duration::from_secs(1));
  g.measurement_time(std::time::Duration::from_secs(2));

  g.bench_function("autoscale_cuckoo_filter", |b| {
    let mut f = CuckooFilterBuilder::new()
      .initial_capacity(CAPACITY)
      .false_positive_probability(FPP)
      .hasher(GxHasher::default())
      .finish::<u64>();
    for i in 0..CAPACITY as u64 {
      f.add_if_not_exist(&i);
    }
    let mut i = 0u64;
    b.iter(|| {
      let r = f.contains(&i);
      i = (i + 1) % CAPACITY as u64;
      black_box(r)
    })
  });

  g.bench_function("cuckoo_filter", |b| {
    let mut f = ScalableCuckooFilter::<u64>::new(CAPACITY, FPP);
    for i in 0..CAPACITY as u64 {
      f.insert_if_not_contained(&i);
    }
    let mut i = 0u64;
    b.iter(|| {
      let r = f.contains(&i);
      i = (i + 1) % CAPACITY as u64;
      black_box(r)
    })
  });

  g.bench_function("cuckoofilter", |b| {
    let mut f: OriginalCuckooFilter<FarmHasher> = OriginalCuckooFilter::with_capacity(CAPACITY);
    for i in 0..CAPACITY as u64 {
      let _ = f.test_and_add(&i);
    }
    let mut i = 0u64;
    b.iter(|| {
      let r = f.contains(&i);
      i = (i + 1) % CAPACITY as u64;
      black_box(r)
    })
  });

  g.finish();
}

fn bench_remove(c: &mut Criterion) {
  let mut g = c.benchmark_group("remove");
  g.sample_size(30);
  g.warm_up_time(std::time::Duration::from_secs(1));
  g.measurement_time(std::time::Duration::from_secs(2));

  g.bench_function("autoscale_cuckoo_filter", |b| {
    b.iter_batched(
      || {
        let mut f = CuckooFilterBuilder::new()
          .initial_capacity(CAPACITY)
          .false_positive_probability(FPP)
          .hasher(GxHasher::default())
          .finish::<u64>();
        for i in 0..CAPACITY as u64 {
          f.add_if_not_exist(&i);
        }
        f
      },
      |mut f: autoscale_cuckoo_filter::CuckooFilter<u64, GxHasher>| {
        for i in 0..50u64 {
          f.remove(&i);
        }
      },
      BatchSize::SmallInput,
    )
  });

  g.bench_function("cuckoo_filter", |b| {
    b.iter_batched(
      || {
        let mut f = ScalableCuckooFilter::<u64>::new(CAPACITY, FPP);
        for i in 0..CAPACITY as u64 {
          f.insert_if_not_contained(&i);
        }
        f
      },
      |mut f: ScalableCuckooFilter<u64>| {
        for i in 0..50u64 {
          f.remove(&i);
        }
      },
      BatchSize::SmallInput,
    )
  });

  g.bench_function("cuckoofilter", |b| {
    b.iter_batched(
      || {
        let mut f: OriginalCuckooFilter<FarmHasher> = OriginalCuckooFilter::with_capacity(CAPACITY);
        for i in 0..CAPACITY as u64 {
          let _ = f.test_and_add(&i);
        }
        f
      },
      |mut f: OriginalCuckooFilter<FarmHasher>| {
        for i in 0..50u64 {
          f.delete(&i);
        }
      },
      BatchSize::SmallInput,
    )
  });

  g.finish();
}

fn bench_memory(c: &mut Criterion) {
  let mut g = c.benchmark_group("memory");
  g.sample_size(20);
  g.warm_up_time(std::time::Duration::from_secs(1));
  g.measurement_time(std::time::Duration::from_secs(1));

  for n in [1_000, 5_000] {
    g.bench_function(BenchmarkId::new("autoscale", n), |b| {
      b.iter_batched(
        || (),
        |_| {
          let mut f = CuckooFilterBuilder::new()
            .initial_capacity(n)
            .false_positive_probability(FPP)
            .hasher(GxHasher::default())
            .finish::<u64>();
          for i in 0..n as u64 {
            f.add_if_not_exist(&i);
          }
          f.bits()
        },
        BatchSize::SmallInput,
      )
    });

    g.bench_function(BenchmarkId::new("scalable", n), |b| {
      b.iter_batched(
        || (),
        |_| {
          let mut f = ScalableCuckooFilter::<u64>::new(n, FPP);
          for i in 0..n as u64 {
            f.insert_if_not_contained(&i);
          }
          f.bits()
        },
        BatchSize::SmallInput,
      )
    });

    g.bench_function(BenchmarkId::new("cuckoo", n), |b| {
      b.iter_batched(
        || (),
        |_| {
          let mut f: OriginalCuckooFilter<FarmHasher> = OriginalCuckooFilter::with_capacity(n);
          for i in 0..n as u64 {
            let _ = f.test_and_add(&i);
          }
          f.memory_usage()
        },
        BatchSize::SmallInput,
      )
    });
  }

  g.finish();
}

criterion_group!(
  benches,
  bench_insert,
  bench_contains,
  bench_remove,
  bench_memory
);

fn main() {
  run_bench_json();
  benches();
  Criterion::default().configure_from_args().final_summary();
}
