// Criterion-based benchmark / 基于 Criterion 的基准测试
// Fixed-duration tests for fair comparison / 固定时长测试以公平对比

use std::{fs::File, io::Write, time::Duration};

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use serde::Serialize;
use std::hint::black_box;
use tikv_jemallocator::Jemalloc;

// Global allocator / 全局分配器
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

mod common;
mod zipf_data;

use common::{BenchConfig, LruBench};
use zipf_data::{ZipfDataConfig, ZipfDataGenerator};

// Random seed / 随机种子
const SEED: u64 = 42;

// Miss key prefix / Miss键前缀
const MISS_PREFIX: &[u8] = b"__MISS__";

// JSON output path / JSON 输出路径
const JSON_PATH: &str = "bench.json";

/// Get thread-local allocated bytes / 获取线程本地已分配字节数
fn get_thread_allocated() -> u64 {
  tikv_jemalloc_ctl::thread::allocatedp::read()
    .unwrap()
    .get()
}

/// Get thread-local deallocated bytes / 获取线程本地已释放字节数
fn get_thread_deallocated() -> u64 {
  tikv_jemalloc_ctl::thread::deallocatedp::read()
    .unwrap()
    .get()
}

#[cfg(feature = "bench-size-lru")]
mod adapter_size_lru;
#[cfg(feature = "bench-clru")]
mod adapter_clru;
#[cfg(feature = "bench-lru")]
mod adapter_lru;
#[cfg(feature = "bench-mini-moka")]
mod adapter_mini_moka;
#[cfg(feature = "bench-moka")]
mod adapter_moka;
#[cfg(feature = "bench-hashlink")]
mod adapter_hashlink;
#[cfg(feature = "bench-schnellru")]
mod adapter_schnellru;

type KvPair = (Vec<u8>, Vec<u8>);

/// Size distribution bucket / 大小分布桶
#[derive(Serialize)]
struct SizeBucket {
  label: String,
  count: usize,
  percent: f64,
  total_size_bytes: u64,
  size_percent: f64,
}

/// Dataset statistics / 数据集统计
#[derive(Serialize)]
struct DatasetStats {
  total_size_bytes: u64,
  item_count: usize,
  avg_item_size: usize,
  min_item_size: usize,
  max_item_size: usize,
  mem_budget: u64,
  size_distribution: Vec<SizeBucket>,
}

/// Benchmark result for JSON output / JSON 输出的基准测试结果
#[derive(Serialize)]
struct BenchResult {
  lib: String,
  hit_rate: f64,
  ops_per_second: f64,
  effective_ops: f64,
  memory_mb: f64,
}

/// Full benchmark output / 完整基准测试输出
#[derive(Serialize)]
struct BenchOutput {
  config: BenchConfig,
  miss_latency_ns: u64,
  miss_latency_method: String,
  stats: DatasetStats,
  results: Vec<BenchResult>,
}

/// Generate a "real miss" key / 生成真实 miss key
#[inline]
fn gen_miss_key(rng: &mut fastrand::Rng, buf: &mut Vec<u8>) {
  buf.clear();
  buf.extend_from_slice(MISS_PREFIX);
  buf.extend_from_slice(&rng.u64(..).to_le_bytes());
}

/// Calculate effective OPS considering miss latency / 计算考虑 miss 延迟的有效 OPS
#[inline]
fn calc_effective_ops(ops_per_second: f64, hit_rate: f64, miss_latency_ns: u64) -> f64 {
  let hit_time_ns = 1e9 / ops_per_second;
  let miss_rate = 1.0 - hit_rate;
  let avg_time_ns = hit_time_ns + miss_rate * miss_latency_ns as f64;
  1e9 / avg_time_ns
}

/// Estimate average item size for hot keys / 估算热点键的平均条目大小
/// Samples using Zipf distribution to get realistic average
/// 使用 Zipf 分布采样以获得真实平均值
fn estimate_hot_avg_size(data: &[(Vec<u8>, Vec<u8>)], generator: &mut ZipfDataGenerator) -> usize {
  let mut total_size = 0u64;
  let samples = 1000; // Smaller sample for speed
  
  for _ in 0..samples {
    let idx = generator.sample_key_index();
    let (k, v) = &data[idx];
    total_size += (k.len() + v.len()) as u64;
  }
  
  (total_size / samples) as usize
}

/// Get calibrated capacity for a library / 获取库的校准容量
/// For size-aware caches: returns bytes / 对于大小感知缓存：返回字节数
/// For count-based caches: returns item count / 对于基于计数的缓存：返回条目数
pub fn get_cache_capacity(
  lib: &str,
  mem_budget: usize,
  avg_item_size: usize,
  scale_factor: f64,
) -> usize {
  // Adjust budget by scale factor / 用缩放系数调整预算
  let adjusted_budget = (mem_budget as f64 / scale_factor) as usize;
  let adjusted_budget = adjusted_budget.max(1024 * 1024); // At least 1MB
  
  // Size-aware caches use bytes directly / 大小感知缓存直接使用字节
  let size_aware = ["size_lru", "moka", "mini_moka"];
  
  if size_aware.contains(&lib) {
    adjusted_budget
  } else {
    // Count-based caches need item count / 基于计数的缓存需要条目数
    // Estimate: budget / (avg_size + overhead)
    // 估算：预算 / (平均大小 + 开销)
    let item_overhead = 96; // Approximate overhead per item
    let total_per_item = avg_item_size + item_overhead;
    (adjusted_budget / total_per_item).max(1)
  }
}

/// Sample memory scaling factor for a cache / 采样缓存的内存缩放系数
/// Benchmark a single cache implementation / 评测单个缓存实现
fn bench_cache<C: LruBench>(
  c: &mut Criterion,
  data: &[KvPair],
  mem_budget: usize,
  avg_item_size: usize,
  scale_factor: f64,
  config: &BenchConfig,
  generator: &mut ZipfDataGenerator,
  results: &mut Vec<BenchResult>,
  miss_latency_ns: u64,
) {
  // Measure baseline memory before cache creation / 创建cache前测量基线内存
  let alloc_before = get_thread_allocated();
  let dealloc_before = get_thread_deallocated();
  
  // Get capacity using scale factor / 使用缩放系数获取容量
  let capacity = get_cache_capacity(C::NAME, mem_budget, avg_item_size, scale_factor);
  
  let mut cache = C::new(capacity, (capacity / 1024 / 1024) as u64);
  let name = C::NAME;
  
  let mut group = c.benchmark_group(name);
  
  // Configure measurement time / 配置测量时间
  group.measurement_time(Duration::from_secs(3));
  group.warm_up_time(Duration::from_millis(500));
  group.sample_size(20);
  
  // Warmup: fill cache / 预热：填充缓存
  for (k, v) in data.iter().take(5000) {
    cache.set(k, v);
  }

  let mut rng = fastrand::Rng::with_seed(SEED);
  let mut miss_key_buf = Vec::with_capacity(16);
  
  // Run workload to measure hit rate / 运行工作负载测量命中率
  let mut total_hits = 0u64;
  let mut total_reads = 0u64;
  
  for _ in 0..100_000 {
    let op = rng.u8(0..100);

    if op < config.read_ratio {
      total_reads += 1;
      if rng.u8(0..100) < config.real_miss_ratio {
        gen_miss_key(&mut rng, &mut miss_key_buf);
        cache.get(&miss_key_buf);
      } else {
        let idx = generator.sample_key_index();
        let (k, _) = &data[idx];
        if cache.get(k) {
          total_hits += 1;
        }
      }
    } else if op < config.read_ratio + config.write_ratio {
      let idx = generator.sample_key_index();
      let (k, v) = &data[idx];
      cache.set(k, v);
    } else {
      let idx = generator.sample_key_index();
      let (k, _) = &data[idx];
      cache.del(k);
    }
  }

  let hit_rate = if total_reads > 0 {
    total_hits as f64 / total_reads as f64
  } else {
    0.0
  };

  // Measure final memory usage using thread-local stats / 使用线程本地统计测量最终内存使用
  let alloc_after = get_thread_allocated();
  let dealloc_after = get_thread_deallocated();
  
  // Net memory = allocated - deallocated / 净内存 = 已分配 - 已释放
  let mem_before = alloc_before.saturating_sub(dealloc_before);
  let mem_after = alloc_after.saturating_sub(dealloc_after);
  let actual_mem = mem_after.saturating_sub(mem_before);
  
  let actual_mem_mb = actual_mem as f64 / (1024.0 * 1024.0);
  
  // Calculate scaling factor / 计算缩放系数
  let scale_factor = if mem_budget > 0 {
    actual_mem as f64 / mem_budget as f64
  } else {
    1.0
  };

  println!(
    "{}: hit_rate={:.2}%, capacity={}, budget={:.1}MB, actual={:.1}MB, scale={:.3}x",
    name,
    hit_rate * 100.0,
    capacity,
    mem_budget as f64 / (1024.0 * 1024.0),
    actual_mem_mb,
    scale_factor
  );

  // Now run performance benchmark / 现在运行性能测试
  group.bench_function(BenchmarkId::new("mixed_workload", name), |b| {
    let mut rng = fastrand::Rng::with_seed(SEED + 1000);
    let mut miss_key_buf = Vec::with_capacity(16);

    b.iter(|| {
      // Run 1000 operations per iteration / 每次迭代运行 1000 次操作
      for _ in 0..1000 {
        let op = rng.u8(0..100);

        if op < config.read_ratio {
          if rng.u8(0..100) < config.real_miss_ratio {
            gen_miss_key(&mut rng, &mut miss_key_buf);
            black_box(cache.get(&miss_key_buf));
          } else {
            let idx = generator.sample_key_index();
            let (k, _) = &data[idx];
            black_box(cache.get(k));
          }
        } else if op < config.read_ratio + config.write_ratio {
          let idx = generator.sample_key_index();
          let (k, v) = &data[idx];
          cache.set(k, v);
        } else {
          let idx = generator.sample_key_index();
          let (k, _) = &data[idx];
          cache.del(k);
        }
      }
    });
  });

  group.finish();

  let ops_per_second = 100_000.0; // Placeholder
  let effective_ops = calc_effective_ops(ops_per_second, hit_rate, miss_latency_ns);

  results.push(BenchResult {
    lib: name.to_string(),
    hit_rate,
    ops_per_second,
    effective_ops,
    memory_mb: actual_mem as f64 / (1024.0 * 1024.0),
  });
}

fn criterion_benchmark(c: &mut Criterion) {
  println!("\n=== Generating Test Data ===\n");

  let config = BenchConfig {
    mem_budget: 200 * 1024 * 1024, // 200MB - base budget
    read_ratio: 90,
    write_ratio: 9,
    delete_ratio: 1,
    real_miss_ratio: 5,
    zipf_s: 1.0,
    ops_per_loop: 0,
    loops: 0,
  };

  let target_mem_mb = (config.mem_budget / 1024 / 1024) as u64;
  let miss_latency_ns = 18_000u64;

  let zipf_config = ZipfDataConfig {
    num_keys: 500_000,
    zipf_s: config.zipf_s,
    seed: SEED,
  };

  let mut generator = ZipfDataGenerator::new(zipf_config);
  let data = generator.generate_all();

  let total_size = ZipfDataGenerator::total_size(&data);
  let avg_size = ZipfDataGenerator::avg_size(&data);
  let (min_size, max_size) = ZipfDataGenerator::size_range(&data);

  println!(
    "Generated {} items, {:.2}MB total",
    data.len(),
    total_size as f64 / (1024.0 * 1024.0)
  );
  println!("Avg size: {}B, Range: {}B - {}B", avg_size, min_size, max_size);
  println!("Memory budget: {}MB\n", target_mem_mb);

  // Estimate hot key average size / 估算热点键平均大小
  let hot_avg_size = estimate_hot_avg_size(&data, &mut generator);
  println!("Hot key avg size: {}B\n", hot_avg_size);

  // Fixed scale factors based on library characteristics / 基于库特性的固定缩放系数
  // Size-aware caches (bytes): ~1.1x overhead
  // Count-based caches: need adjustment based on item size
  // 大小感知缓存（字节）：约 1.1x 开销
  // 基于计数的缓存：需要根据条目大小调整
  let scale_factors = vec![
    ("size_lru", 1.1),
    ("moka", 1.1),
    ("mini_moka", 1.1),
    ("clru", 1.0),
    ("lru", 1.0),
    ("hashlink", 1.0),
    ("schnellru", 1.0),
  ];

  // Calculate size distribution / 计算大小分布
  let mut size_buckets = vec![(0usize, 0u64); 5];
  for (k, v) in &data {
    let size = k.len() + v.len();
    let bucket_idx = if size < 100 {
      0 // <100B (Tiny)
    } else if size < 1024 {
      1 // 100B-1KB (Small)
    } else if size < 10240 {
      2 // 1-10KB (Medium)
    } else if size < 102400 {
      3 // 10-100KB (Large)
    } else {
      4 // >=100KB (Huge)
    };
    size_buckets[bucket_idx].0 += 1;
    size_buckets[bucket_idx].1 += size as u64;
  }

  let size_distribution: Vec<SizeBucket> = size_buckets
    .iter()
    .enumerate()
    .filter(|(_, (count, _))| *count > 0)
    .map(|(i, &(count, bucket_size))| {
      let labels = [
        "<100B", "100B-1KB", "1-10KB", "10-100KB", ">=100KB",
      ];
      SizeBucket {
        label: labels[i].to_string(),
        count,
        percent: count as f64 / data.len() as f64 * 100.0,
        total_size_bytes: bucket_size,
        size_percent: bucket_size as f64 / total_size as f64 * 100.0,
      }
    })
    .collect();

  let stats = DatasetStats {
    total_size_bytes: total_size,
    item_count: data.len(),
    avg_item_size: avg_size,
    min_item_size: min_size,
    max_item_size: max_size,
    mem_budget: config.mem_budget as u64,
    size_distribution,
  };

  let mem_budget = config.mem_budget;
  let mut results = Vec::new();

  // Helper to get scale factor / 获取缩放系数的辅助函数
  let get_scale = |name: &str| -> f64 {
    scale_factors
      .iter()
      .find(|(n, _)| *n == name)
      .map(|(_, s)| *s)
      .unwrap_or(1.0)
  };

  // Benchmark each implementation / 评测每个实现
  #[cfg(feature = "bench-size-lru")]
  bench_cache::<adapter_size_lru::SizeLruAdapter>(
    c,
    &data,
    mem_budget,
    avg_size,
    get_scale("size_lru"),
    &config,
    &mut generator,
    &mut results,
    miss_latency_ns,
  );

  #[cfg(feature = "bench-moka")]
  bench_cache::<adapter_moka::MokaAdapter>(
    c,
    &data,
    mem_budget,
    avg_size,
    get_scale("moka"),
    &config,
    &mut generator,
    &mut results,
    miss_latency_ns,
  );

  #[cfg(feature = "bench-mini-moka")]
  bench_cache::<adapter_mini_moka::MiniMokaAdapter>(
    c,
    &data,
    mem_budget,
    avg_size,
    get_scale("mini_moka"),
    &config,
    &mut generator,
    &mut results,
    miss_latency_ns,
  );

  #[cfg(feature = "bench-clru")]
  bench_cache::<adapter_clru::ClruAdapter>(
    c,
    &data,
    mem_budget,
    avg_size,
    get_scale("clru"),
    &config,
    &mut generator,
    &mut results,
    miss_latency_ns,
  );

  #[cfg(feature = "bench-lru")]
  bench_cache::<adapter_lru::LruAdapter>(
    c,
    &data,
    mem_budget,
    avg_size,
    get_scale("lru"),
    &config,
    &mut generator,
    &mut results,
    miss_latency_ns,
  );

  #[cfg(feature = "bench-hashlink")]
  bench_cache::<adapter_hashlink::HashlinkAdapter>(
    c,
    &data,
    mem_budget,
    avg_size,
    get_scale("hashlink"),
    &config,
    &mut generator,
    &mut results,
    miss_latency_ns,
  );

  #[cfg(feature = "bench-schnellru")]
  bench_cache::<adapter_schnellru::SchnellruAdapter>(
    c,
    &data,
    mem_budget,
    avg_size,
    get_scale("schnellru"),
    &config,
    &mut generator,
    &mut results,
    miss_latency_ns,
  );

  // Sort by effective_ops descending / 按有效 OPS 降序排序
  results.sort_by(|a, b| b.effective_ops.partial_cmp(&a.effective_ops).unwrap());

  // Write JSON output / 写入 JSON 输出
  let output = BenchOutput {
    config,
    miss_latency_ns,
    miss_latency_method: "DapuStor X5900 PCIe 5.0 NVMe (18µs)".to_string(),
    stats,
    results,
  };

  let json = sonic_rs::to_string_pretty(&output).expect("JSON serialize");
  let mut file = File::create(JSON_PATH).expect("create bench.json");
  file.write_all(json.as_bytes()).expect("write bench.json");

  println!("\nResults written to {JSON_PATH}");
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
