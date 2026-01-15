// Main benchmark file
// 主评测文件

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use rand::Rng;
use std::collections::HashSet;
use std::hint::black_box;
use std::time::Duration;

mod r#trait;
use r#trait::FilterBench;

// jdb_xorf implementations
// jdb_xorf 实现
#[cfg(feature = "bench-jdb")]
mod jdb_impl {
  use super::FilterBench;
  use jdb_xorf::{BinaryFuse8, BinaryFuse16, BinaryFuse32, Filter};

  pub struct JdbBinaryFuse8(BinaryFuse8);
  pub struct JdbBinaryFuse16(BinaryFuse16);
  pub struct JdbBinaryFuse32(BinaryFuse32);

  impl FilterBench for JdbBinaryFuse8 {
    const NAME: &'static str = "jdb_BinaryFuse8";

    fn build(keys: &[u64]) -> Self {
      Self(BinaryFuse8::try_from(keys).unwrap())
    }

    fn contains(&self, key: &u64) -> bool {
      self.0.contains(key)
    }
  }

  impl FilterBench for JdbBinaryFuse16 {
    const NAME: &'static str = "jdb_BinaryFuse16";

    fn build(keys: &[u64]) -> Self {
      Self(BinaryFuse16::try_from(keys).unwrap())
    }

    fn contains(&self, key: &u64) -> bool {
      self.0.contains(key)
    }
  }

  impl FilterBench for JdbBinaryFuse32 {
    const NAME: &'static str = "jdb_BinaryFuse32";

    fn build(keys: &[u64]) -> Self {
      Self(BinaryFuse32::try_from(keys).unwrap())
    }

    fn contains(&self, key: &u64) -> bool {
      self.0.contains(key)
    }
  }
}

// xorf implementations
// xorf 实现
#[cfg(feature = "bench-xorf")]
mod xorf_impl {
  use super::FilterBench;
  use xorf::{BinaryFuse8, BinaryFuse16, BinaryFuse32, Filter};

  pub struct XorfBinaryFuse8(BinaryFuse8);
  pub struct XorfBinaryFuse16(BinaryFuse16);
  pub struct XorfBinaryFuse32(BinaryFuse32);

  impl FilterBench for XorfBinaryFuse8 {
    const NAME: &'static str = "xorf_BinaryFuse8";

    fn build(keys: &[u64]) -> Self {
      Self(BinaryFuse8::try_from(keys).unwrap())
    }

    fn contains(&self, key: &u64) -> bool {
      self.0.contains(key)
    }
  }

  impl FilterBench for XorfBinaryFuse16 {
    const NAME: &'static str = "xorf_BinaryFuse16";

    fn build(keys: &[u64]) -> Self {
      Self(BinaryFuse16::try_from(keys).unwrap())
    }

    fn contains(&self, key: &u64) -> bool {
      self.0.contains(key)
    }
  }

  impl FilterBench for XorfBinaryFuse32 {
    const NAME: &'static str = "xorf_BinaryFuse32";

    fn build(keys: &[u64]) -> Self {
      Self(BinaryFuse32::try_from(keys).unwrap())
    }

    fn contains(&self, key: &u64) -> bool {
      self.0.contains(key)
    }
  }
}

fn gen_keys(n: usize) -> Vec<u64> {
  let mut rng = rand::rng();
  let mut set = HashSet::new();
  while set.len() < n {
    set.insert(rng.random());
  }
  set.into_iter().collect()
}

fn bench_build<F: FilterBench>(c: &mut Criterion, size: usize) {
  let keys = gen_keys(size);
  let id = BenchmarkId::new(F::NAME, size);

  let mut group = c.benchmark_group("build");
  group.warm_up_time(Duration::from_millis(100));
  group.measurement_time(Duration::from_secs(1));
  group.throughput(Throughput::Elements(size as u64));
  group.bench_with_input(id, &keys, |b, keys| {
    b.iter(|| {
      let filter = F::build(black_box(keys));
      black_box(filter);
    });
  });
  group.finish();
}

fn bench_contains<F: FilterBench>(c: &mut Criterion, size: usize) {
  let keys = gen_keys(size);
  let filter = F::build(&keys);
  let id = BenchmarkId::new(F::NAME, size);

  let mut group = c.benchmark_group("contains");
  group.warm_up_time(Duration::from_millis(100));
  group.measurement_time(Duration::from_secs(1));
  group.throughput(Throughput::Elements(size as u64));
  group.bench_with_input(id, &filter, |b, filter| {
    b.iter(|| {
      for key in &keys {
        black_box(filter.contains(black_box(key)));
      }
    });
  });
  group.finish();
}

fn bench_false_positive<F: FilterBench>(c: &mut Criterion, size: usize) {
  let keys = gen_keys(size);
  let key_set: HashSet<_> = keys.iter().copied().collect();
  let filter = F::build(&keys);
  let test_keys = gen_keys(size * 10);
  let id = BenchmarkId::new(F::NAME, size);

  let mut group = c.benchmark_group("false_positive");
  group.warm_up_time(std::time::Duration::from_millis(100));
  group.measurement_time(std::time::Duration::from_secs(1));
  group.throughput(Throughput::Elements((size * 10) as u64));
  group.bench_with_input(id, &(&filter, &test_keys, &key_set), |b, (filter, test_keys, key_set)| {
    b.iter(|| {
      let mut fp_count = 0;
      for key in *test_keys {
        if filter.contains(black_box(key)) && !key_set.contains(key) {
          fp_count += 1;
        }
      }
      black_box(fp_count)
    });
  });
  group.finish();
}

fn bench_false_negative<F: FilterBench>(c: &mut Criterion, size: usize) {
  let keys = gen_keys(size);
  let filter = F::build(&keys);
  let id = BenchmarkId::new(F::NAME, size);

  let mut group = c.benchmark_group("false_negative");
  group.warm_up_time(std::time::Duration::from_millis(100));
  group.measurement_time(std::time::Duration::from_secs(1));
  group.throughput(Throughput::Elements(size as u64));
  group.bench_with_input(id, &(&filter, &keys), |b, (filter, keys)| {
    b.iter(|| {
      let mut fn_count = 0;
      for key in *keys {
        if !filter.contains(black_box(key)) {
          fn_count += 1;
        }
      }
      black_box(fn_count)
    });
  });
  group.finish();
}

macro_rules! bench_all {
  ($c:expr, $size:expr, $($filter:ty),+) => {
    $(
      bench_build::<$filter>($c, $size);
      bench_contains::<$filter>($c, $size);
      bench_false_positive::<$filter>($c, $size);
      bench_false_negative::<$filter>($c, $size);
    )+
  };
}

fn benchmark_filters(c: &mut Criterion) {
  let sizes = [1000, 10000, 100000];

  for size in sizes {
    #[cfg(feature = "bench-jdb")]
    {
      bench_all!(
        c,
        size,
        jdb_impl::JdbBinaryFuse8,
        jdb_impl::JdbBinaryFuse16,
        jdb_impl::JdbBinaryFuse32
      );
    }

    #[cfg(feature = "bench-xorf")]
    {
      bench_all!(
        c,
        size,
        xorf_impl::XorfBinaryFuse8,
        xorf_impl::XorfBinaryFuse16,
        xorf_impl::XorfBinaryFuse32
      );
    }
  }
}

criterion_group!(benches, benchmark_filters);
criterion_main!(benches);
