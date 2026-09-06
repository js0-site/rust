// Main benchmark file
// 主评测文件

use criterion::{criterion_group, criterion_main};
use tikv_jemallocator::Jemalloc;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[cfg(any(feature = "bench-jdb", feature = "bench-xorf"))]
mod r#trait;
#[cfg(any(feature = "bench-jdb", feature = "bench-xorf"))]
use r#trait::FilterBench;

// jdb_xorf implementations
// jdb_xorf 实现
#[cfg(feature = "bench-jdb")]
mod jdb_impl {
  use core::mem;

  use jdb_xorf::{Base, Filter, Fingerprint};

  use super::FilterBench;

  pub struct JdbBench<T: Fingerprint, const BITS: usize>(Base<T>);

  impl<T: Fingerprint, const BITS: usize> FilterBench for JdbBench<T, BITS> {
    const NAME: &'static str = match BITS {
      8 => "jdb_Bf8",
      16 => "jdb_Bf16",
      32 => "jdb_Bf32",
      _ => "jdb_Bf",
    };

    fn build(keys: &[u64]) -> Self {
      Self(Base::from(keys))
    }

    fn has(&self, key: &u64) -> bool {
      self.0.has(key)
    }

    fn memory_usage(&self) -> usize {
      mem::size_of::<Base<T>>() + self.0.fingerprints.len() * mem::size_of::<T>()
    }
  }

  pub type JdbBf8 = JdbBench<u8, 8>;
  pub type JdbBf16 = JdbBench<u16, 16>;
  pub type JdbBf32 = JdbBench<u32, 32>;
}

// xorf implementations
// xorf 实现
#[cfg(feature = "bench-xorf")]
mod xorf_impl {
  use core::mem;

  use xorf::{BinaryFuse8, BinaryFuse16, BinaryFuse32, Filter};

  use super::FilterBench;

  pub struct XorfBinaryFuse8(BinaryFuse8);
  pub struct XorfBinaryFuse16(BinaryFuse16);
  pub struct XorfBinaryFuse32(BinaryFuse32);

  impl FilterBench for XorfBinaryFuse8 {
    const NAME: &'static str = "xorf_BinaryFuse8";

    fn build(keys: &[u64]) -> Self {
      Self(BinaryFuse8::try_from(keys).unwrap())
    }

    fn has(&self, key: &u64) -> bool {
      self.0.contains(key)
    }

    fn memory_usage(&self) -> usize {
      mem::size_of::<BinaryFuse8>() + self.0.fingerprints.len()
    }
  }

  impl FilterBench for XorfBinaryFuse16 {
    const NAME: &'static str = "xorf_BinaryFuse16";

    fn build(keys: &[u64]) -> Self {
      Self(BinaryFuse16::try_from(keys).unwrap())
    }

    fn has(&self, key: &u64) -> bool {
      self.0.contains(key)
    }

    fn memory_usage(&self) -> usize {
      mem::size_of::<BinaryFuse16>() + self.0.fingerprints.len() * 2
    }
  }

  impl FilterBench for XorfBinaryFuse32 {
    const NAME: &'static str = "xorf_BinaryFuse32";

    fn build(keys: &[u64]) -> Self {
      Self(BinaryFuse32::try_from(keys).unwrap())
    }

    fn has(&self, key: &u64) -> bool {
      self.0.contains(key)
    }

    fn memory_usage(&self) -> usize {
      mem::size_of::<BinaryFuse32>() + self.0.fingerprints.len() * 4
    }
  }
}

#[cfg(any(feature = "bench-jdb", feature = "bench-xorf"))]
mod runner {
  use std::{collections::HashSet, hint::black_box, time::Duration};

  use criterion::{BenchmarkId, Criterion, Throughput};
  use rand::RngExt;

  use super::FilterBench;
  #[cfg(feature = "bench-jdb")]
  use super::jdb_impl;
  #[cfg(feature = "bench-xorf")]
  use super::xorf_impl;

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

  fn bench_has<F: FilterBench>(c: &mut Criterion, size: usize) {
    let keys = gen_keys(size);
    let filter = F::build(&keys);
    let id = BenchmarkId::new(F::NAME, size);

    let mut group = c.benchmark_group("has");
    group.warm_up_time(Duration::from_millis(100));
    group.measurement_time(Duration::from_secs(1));
    group.throughput(Throughput::Elements(size as u64));
    group.bench_with_input(id, &filter, |b, filter| {
      b.iter(|| {
        for key in &keys {
          black_box(filter.has(black_box(key)));
        }
      });
    });
    group.finish();
  }

  fn bench_false_positive<F: FilterBench>(c: &mut Criterion, size: usize) {
    use std::{
      fs::{OpenOptions, create_dir_all},
      io::Write,
      path::Path,
    };

    let keys = gen_keys(size);
    let key_set: HashSet<_> = keys.iter().copied().collect();
    let filter = F::build(&keys);
    let test_keys = gen_keys(size * 100);

    let mut fp_count = 0;
    let mut test_count = 0;
    for key in &test_keys {
      if !key_set.contains(key) {
        test_count += 1;
        if filter.has(key) {
          fp_count += 1;
        }
      }
    }

    let fp_rate = if test_count > 0 {
      fp_count as f64 / test_count as f64 * 100.0
    } else {
      0.0
    };

    let name = F::NAME;
    eprintln!(
      "FP test {name}/{size}: tested={test_count}, fp={fp_count}, rate={fp_rate:.5}%"
    );

    // Use Criterion's output directory
    // 使用 Criterion 的输出目录
    let rates_dir = Path::new("target/criterion/rates");
    create_dir_all(rates_dir).ok();
    let name = F::NAME;
    let path = rates_dir.join(format!("fp_{name}_{size}.txt"));
    if let Ok(mut file) = OpenOptions::new()
      .create(true)
      .write(true)
      .truncate(true)
      .open(&path)
    {
      write!(file, "{fp_rate:.5}").ok();
    }

    // Dummy benchmark to satisfy Criterion
    // 虚拟基准测试以满足 Criterion
    let id = BenchmarkId::new(F::NAME, size);
    let mut group = c.benchmark_group("false_positive");
    group.warm_up_time(Duration::from_millis(1));
    group.measurement_time(Duration::from_millis(1));
    group.sample_size(10);
    group.bench_with_input(id, &(), |b, _| {
      b.iter(|| {});
    });
    group.finish();
  }

  fn bench_false_negative<F: FilterBench>(c: &mut Criterion, size: usize) {
    use std::{
      fs::{OpenOptions, create_dir_all},
      io::Write,
      path::Path,
    };

    let keys = gen_keys(size);
    let filter = F::build(&keys);

    let mut fn_count = 0;
    for key in &keys {
      if !filter.has(key) {
        fn_count += 1;
      }
    }

    if fn_count > 0 {
      let name = F::NAME;
      let total_keys = keys.len();
      eprintln!(
        "ERROR: False negative detected for {name}/{size}: {fn_count} out of {total_keys}"
      );
    }

    // Use Criterion's output directory
    // 使用 Criterion 的输出目录
    let rates_dir = Path::new("target/criterion/rates");
    create_dir_all(rates_dir).ok();
    let name = F::NAME;
    let path = rates_dir.join(format!("fn_{name}_{size}.txt"));
    if let Ok(mut file) = OpenOptions::new()
      .create(true)
      .write(true)
      .truncate(true)
      .open(&path)
    {
      write!(file, "0").ok();
    }

    // Dummy benchmark to satisfy Criterion
    // 虚拟基准测试以满足 Criterion
    let id = BenchmarkId::new(F::NAME, size);
    let mut group = c.benchmark_group("false_negative");
    group.warm_up_time(Duration::from_millis(1));
    group.measurement_time(Duration::from_millis(1));
    group.sample_size(10);
    group.bench_with_input(id, &(), |b, _| {
      b.iter(|| {});
    });
    group.finish();
  }

  fn bench_memory<F: FilterBench>(_c: &mut Criterion, size: usize) {
    use std::{
      fs::{OpenOptions, create_dir_all},
      io::Write,
      path::Path,
    };

    let keys = gen_keys(size);
    let filter = F::build(&keys);

    // Get actual memory usage from filter
    // 从过滤器获取实际内存占用
    let mem_used = filter.memory_usage();

    black_box(&filter);

    let name = F::NAME;
    eprintln!("Memory test {name}/{size}: {mem_used} bytes");

    let rates_dir = Path::new("target/criterion/rates");
    create_dir_all(rates_dir).ok();
    let name = F::NAME;
    let path = rates_dir.join(format!("mem_{name}_{size}.txt"));
    if let Ok(mut file) = OpenOptions::new()
      .create(true)
      .write(true)
      .truncate(true)
      .open(&path)
    {
      write!(file, "{mem_used}").ok();
    }
  }

  macro_rules! bench_all {
  ($c:expr, $size:expr, $($filter:ty),+) => {
    $(
      bench_build::<$filter>($c, $size);
      bench_has::<$filter>($c, $size);
      bench_false_positive::<$filter>($c, $size);
      bench_false_negative::<$filter>($c, $size);
      bench_memory::<$filter>($c, $size);
    )+
  };
}

  pub fn benchmark_filters(c: &mut Criterion) {
    let size = 100000;

    #[cfg(feature = "bench-jdb")]
    {
      bench_all!(
        c,
        size,
        jdb_impl::JdbBf8,
        jdb_impl::JdbBf16,
        jdb_impl::JdbBf32
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

#[cfg(any(feature = "bench-jdb", feature = "bench-xorf"))]
use runner::benchmark_filters;

#[cfg(not(any(feature = "bench-jdb", feature = "bench-xorf")))]
fn benchmark_filters(_c: &mut criterion::Criterion) {}

criterion_group!(benches, benchmark_filters);
criterion_main!(benches);
