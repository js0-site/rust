// Criterion-based benchmark / 基于 Criterion 的基准测试
// Fixed-duration tests for fair comparison / 固定时长测试以公平对比

use std::{fs::File, hint::black_box, io::Write, time::Duration};

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use serde::{Deserialize, Serialize};
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

// JSON output path / JSON 输出路径
const JSON_PATH: &str = "bench.json";

/// Get thread-local allocated bytes / 获取线程本地已分配字节数
fn get_thread_allocated() -> u64 {
  tikv_jemalloc_ctl::thread::allocatedp::read().unwrap().get()
}

/// Get thread-local deallocated bytes / 获取线程本地已释放字节数
fn get_thread_deallocated() -> u64 {
  tikv_jemalloc_ctl::thread::deallocatedp::read()
    .unwrap()
    .get()
}

#[cfg(feature = "bench-clru")]
mod adapter_clru;
#[cfg(feature = "bench-hashlink")]
mod adapter_hashlink;
#[cfg(feature = "bench-lru")]
mod adapter_lru;
#[cfg(feature = "bench-mini-moka")]
mod adapter_mini_moka;
#[cfg(feature = "bench-moka")]
mod adapter_moka;
#[cfg(feature = "bench-schnellru")]
mod adapter_schnellru;
#[cfg(feature = "bench-size-lru")]
mod adapter_size_lru;

type KvPair = (Vec<u8>, Vec<u8>);

/// Size distribution bucket / 大小分布桶
#[derive(Serialize, Deserialize)]
struct SizeBucket {
  label: String,
  count: usize,
  percent: f64,
  total_size_bytes: u64,
  size_percent: f64,
}

/// Dataset statistics / 数据集统计
#[derive(Serialize, Deserialize)]
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
#[derive(Serialize, Deserialize)]
struct BenchResult {
  lib: String,
  hit_rate: f64,
  ops_per_second: f64,
  effective_ops: f64,
  memory_mb: f64,
  capacity: usize,   // Cache capacity used / 使用的缓存容量
  scale_factor: f64, // Scale factor applied / 应用的缩放系数
}

/// Full benchmark output / 完整基准测试输出
#[derive(Serialize, Deserialize)]
struct BenchOutput {
  config: BenchConfig,
  miss_latency_ns: u64,
  miss_latency_method: String,
  stats: DatasetStats,
  results: Vec<BenchResult>,
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

/// Load previous benchmark results for capacity calibration / 加载上次基准测试结果用于容量校准
fn load_previous_results() -> Option<BenchOutput> {
  std::fs::read_to_string(JSON_PATH)
    .ok()
    .and_then(|content| sonic_rs::from_str(&content).ok())
}

/// Calculate scale factor for a library based on previous results / 根据上次结果计算库的缩放系数
fn calculate_scale_factor(lib_name: &str, target_mb: f64, previous_results: &[BenchResult]) -> f64 {
  if let Some(prev_result) = previous_results.iter().find(|r| r.lib == lib_name) {
    let raw_scale = target_mb / prev_result.memory_mb;

    // Apply more aggressive scaling for libraries that are far from target
    // 对远离目标的库应用更激进的缩放
    let adjusted_scale = if prev_result.memory_mb > target_mb * 1.5 {
      // If memory is >1.5x target, be more aggressive
      // 如果内存超过目标1.5倍，更激进
      raw_scale * 0.8
    } else if prev_result.memory_mb < target_mb * 0.5 {
      // If memory is <0.5x target, be more aggressive in the other direction
      // 如果内存低于目标0.5倍，向另一个方向更激进
      raw_scale * 1.5
    } else {
      raw_scale
    };

    println!(
      "  {lib_name}: prev_memory={:.1}MB, target={target_mb}MB, raw_scale={raw_scale:.3}, adjusted_scale={adjusted_scale:.3}",
      prev_result.memory_mb
    );
    adjusted_scale
  } else {
    println!("  {lib_name}: no previous data, using default scale=1.0");
    1.0
  }
}

/// Get calibrated capacity for a library / 获取库的校准容量
/// For size-aware caches: returns bytes / 对于大小感知缓存：返回字节数
/// For count-based caches: returns item count / 对于基于计数的缓存：返回条目数
fn get_cache_capacity(
  lib: &str,
  mem_budget: usize,
  avg_item_size: usize,
  previous_results: Option<&[BenchResult]>,
) -> usize {
  let target_mb = (mem_budget / 1024 / 1024) as f64;

  // Calculate scale factor from previous results if available / 如果有历史结果则计算缩放系数
  let scale_factor = if let Some(results) = previous_results {
    calculate_scale_factor(lib, target_mb, results)
  } else {
    1.0 // First run, use default / 首次运行，使用默认值
  };

  match lib {
    // Size-aware caches: use optimized defaults or scale from previous results
    // 大小感知缓存：使用优化的默认值或从历史结果缩放
    "size_lru" => {
      if let Some(results) = previous_results {
        if let Some(prev_result) = results.iter().find(|r| r.lib == lib) {
          let scaled_capacity = ((prev_result.capacity as f64) * scale_factor) as usize;
          println!(
            "  {lib}: prev_capacity={}, scaled={scaled_capacity}",
            prev_result.capacity
          );
          scaled_capacity
        } else {
          55965315 // Optimized default capacity for ~64MB
        }
      } else {
        55965315 // Optimized default capacity for ~64MB
      }
    }
    "moka" => {
      if let Some(results) = previous_results {
        if let Some(prev_result) = results.iter().find(|r| r.lib == lib) {
          let scaled_capacity = ((prev_result.capacity as f64) * scale_factor) as usize;
          println!(
            "  {lib}: prev_capacity={}, scaled={scaled_capacity}",
            prev_result.capacity
          );
          scaled_capacity
        } else {
          52824664 // Optimized default capacity for ~64MB
        }
      } else {
        52824664 // Optimized default capacity for ~64MB
      }
    }
    "mini-moka" => {
      if let Some(results) = previous_results {
        if let Some(prev_result) = results.iter().find(|r| r.lib == lib) {
          let scaled_capacity = ((prev_result.capacity as f64) * scale_factor) as usize;
          println!(
            "  {lib}: prev_capacity={}, scaled={scaled_capacity}",
            prev_result.capacity
          );
          scaled_capacity.max(1000) // Minimum capacity
        } else {
          55855206 // Optimized default capacity for ~64MB
        }
      } else {
        55855206 // Optimized default capacity for ~64MB
      }
    }

    // Count-based caches: use optimized defaults or scale from previous results
    // 基于计数的缓存：使用优化的默认值或从历史结果缩放
    "lru" => {
      if let Some(results) = previous_results {
        if let Some(prev_result) = results.iter().find(|r| r.lib == lib) {
          let scaled_capacity = ((prev_result.capacity as f64) * scale_factor) as usize;
          println!(
            "  {lib}: prev_capacity={}, scaled={scaled_capacity}",
            prev_result.capacity
          );
          scaled_capacity.max(100) // Minimum capacity
        } else {
          664 // Optimized default capacity for ~64MB
        }
      } else {
        664 // Optimized default capacity for ~64MB
      }
    }
    "hashlink" => {
      if let Some(results) = previous_results {
        if let Some(prev_result) = results.iter().find(|r| r.lib == lib) {
          let scaled_capacity = ((prev_result.capacity as f64) * scale_factor) as usize;
          println!(
            "  {lib}: prev_capacity={}, scaled={scaled_capacity}",
            prev_result.capacity
          );
          scaled_capacity.max(100) // Minimum capacity
        } else {
          678 // Optimized default capacity for ~64MB
        }
      } else {
        678 // Optimized default capacity for ~64MB
      }
    }
    "clru" => {
      if let Some(results) = previous_results {
        if let Some(prev_result) = results.iter().find(|r| r.lib == lib) {
          let scaled_capacity = ((prev_result.capacity as f64) * scale_factor) as usize;
          println!(
            "  {lib}: prev_capacity={}, scaled={scaled_capacity}",
            prev_result.capacity
          );
          scaled_capacity.max(100) // Minimum capacity
        } else {
          55050267 // Optimized default capacity for ~64MB
        }
      } else {
        55050267 // Optimized default capacity for ~64MB
      }
    }
    "schnellru" => {
      if let Some(results) = previous_results {
        if let Some(prev_result) = results.iter().find(|r| r.lib == lib) {
          let scaled_capacity = ((prev_result.capacity as f64) * scale_factor) as usize;
          println!(
            "  {lib}: prev_capacity={}, scaled={scaled_capacity}",
            prev_result.capacity
          );
          scaled_capacity.max(100) // Minimum capacity
        } else {
          702 // Optimized default capacity for ~64MB
        }
      } else {
        702 // Optimized default capacity for ~64MB
      }
    }

    // Fallback for unknown libraries / 未知库的回退处理
    _ => {
      let overhead = 500; // Default overhead
      if let Some(results) = previous_results {
        if let Some(prev_result) = results.iter().find(|r| r.lib == lib) {
          let scaled_capacity = ((prev_result.capacity as f64) * scale_factor) as usize;
          println!(
            "  {lib}: prev_capacity={}, scaled={scaled_capacity}",
            prev_result.capacity
          );
          scaled_capacity.max(100) // Minimum capacity
        } else {
          mem_budget / (avg_item_size + overhead) // Estimate based on memory budget
        }
      } else {
        mem_budget / (avg_item_size + overhead) // Estimate based on memory budget
      }
    }
  }
}

/// Pre-generated operation sequence / 预生成的操作序列
struct OpSequence {
  read_indices: Vec<usize>,   // Indices for read operations / 读操作的索引
  write_indices: Vec<usize>,  // Indices for write operations / 写操作的索引
  delete_indices: Vec<usize>, // Indices for delete operations / 删除操作的索引
  miss_indices: Vec<usize>, // Indices for miss operations (out of range) / Miss操作的索引（超出范围）
}

impl OpSequence {
  /// Generate operation sequence / 生成操作序列
  fn new(
    data_len: usize,
    config: &BenchConfig,
    generator: &mut ZipfDataGenerator,
    ops_count: usize,
  ) -> Self {
    let mut rng = fastrand::Rng::with_seed(SEED);
    let mut read_indices = Vec::new();
    let mut write_indices = Vec::new();
    let mut delete_indices = Vec::new();
    let mut miss_indices = Vec::new();

    for _ in 0..ops_count {
      let op = rng.u8(0..100);

      if op < config.read_ratio {
        if rng.u8(0..100) < config.real_miss_ratio {
          // Generate miss index (out of data range) / 生成miss索引（超出数据范围）
          let miss_idx = data_len + rng.usize(0..1000);
          miss_indices.push(miss_idx);
        } else {
          // Generate read index / 生成读索引
          let idx = generator.sample_key_index();
          read_indices.push(idx);
        }
      } else if op < config.read_ratio + config.write_ratio {
        // Generate write index / 生成写索引
        let idx = generator.sample_key_index();
        write_indices.push(idx);
      } else {
        // Generate delete index / 生成删除索引
        let idx = generator.sample_key_index();
        delete_indices.push(idx);
      }
    }

    Self {
      read_indices,
      write_indices,
      delete_indices,
      miss_indices,
    }
  }
}

/// Benchmark parameters / 评测参数
struct BenchParams<'a> {
  mem_budget: usize,
  avg_item_size: usize,
  previous_results: Option<&'a [BenchResult]>,
  config: &'a BenchConfig,
  miss_latency_ns: u64,
}

/// Benchmark a single cache implementation / 评测单个缓存实现
fn bench_cache<C: LruBench>(
  c: &mut Criterion,
  data: &[KvPair],
  params: &BenchParams,
  generator: &mut ZipfDataGenerator,
  results: &mut Vec<BenchResult>,
) {
  let mem_budget = params.mem_budget;
  let avg_item_size = params.avg_item_size;
  let previous_results = params.previous_results;
  let config = params.config;
  let miss_latency_ns = params.miss_latency_ns;
  let target_mem_mb = (mem_budget / 1024 / 1024) as u64;

  // Get capacity with automatic scaling / 获取自动缩放的容量
  let capacity = get_cache_capacity(C::NAME, mem_budget, avg_item_size, previous_results);

  let scale_factor = if let Some(prev_results) = previous_results {
    calculate_scale_factor(C::NAME, target_mem_mb as f64, prev_results)
  } else {
    1.0
  };

  let scale_info = if previous_results.is_some() {
    format!(" (scale: {scale_factor:.3})")
  } else {
    " (first run)".to_string()
  };

  println!("Testing {}: capacity={}{scale_info}", C::NAME, capacity);

  // Measure memory before cache creation / 创建cache前测量内存
  let alloc_before = get_thread_allocated();
  let dealloc_before = get_thread_deallocated();

  let mut cache = C::new(capacity, target_mem_mb);
  let name = C::NAME;

  let mut group = c.benchmark_group(name);

  // Configure measurement time / 配置测量时间
  group.measurement_time(Duration::from_secs(3));
  group.warm_up_time(Duration::from_millis(500));
  group.sample_size(20);

  // Warmup: fill cache with more data / 预热：用更多数据填充缓存
  for (i, (_, v)) in data.iter().enumerate().take(15000) {
    let key = i.to_le_bytes();
    cache.set(&key, v);
  }

  // Pre-generate operation sequences / 预生成操作序列
  let hit_rate_ops = OpSequence::new(data.len(), config, generator, 100_000);
  let perf_ops = OpSequence::new(data.len(), config, generator, 100_000);
  let bench_ops = OpSequence::new(data.len(), config, generator, 10_000);

  // Run workload to measure hit rate / 运行工作负载测量命中率
  let mut total_hits = 0u64;
  let mut total_reads = 0u64;
  let mut read_idx = 0;
  let mut write_idx = 0;
  let mut delete_idx = 0;
  let mut miss_idx = 0;

  let mut rng = fastrand::Rng::with_seed(SEED);

  for _ in 0..100_000 {
    let op = rng.u8(0..100);

    if op < config.read_ratio {
      total_reads += 1;
      if rng.u8(0..100) < config.real_miss_ratio {
        if miss_idx < hit_rate_ops.miss_indices.len() {
          let miss_data_idx = hit_rate_ops.miss_indices[miss_idx];
          let key = miss_data_idx.to_le_bytes();
          cache.get(&key);
          miss_idx += 1;
        }
      } else {
        if read_idx < hit_rate_ops.read_indices.len() {
          let data_idx = hit_rate_ops.read_indices[read_idx];
          let key = data_idx.to_le_bytes();
          if cache.get(&key) {
            total_hits += 1;
          } else {
            // Cache miss: refill from data source / 缓存miss：从数据源重新填充
            let (_, v) = &data[data_idx];
            cache.set(&key, v);
          }
          read_idx += 1;
        }
      }
    } else if op < config.read_ratio + config.write_ratio {
      if write_idx < hit_rate_ops.write_indices.len() {
        let data_idx = hit_rate_ops.write_indices[write_idx];
        let key = data_idx.to_le_bytes();
        let (_, v) = &data[data_idx];
        cache.set(&key, v);
        write_idx += 1;
      }
    } else {
      if delete_idx < hit_rate_ops.delete_indices.len() {
        let data_idx = hit_rate_ops.delete_indices[delete_idx];
        let key = data_idx.to_le_bytes();
        cache.del(&key);
        delete_idx += 1;
      }
    }
  }

  let hit_rate = if total_reads > 0 {
    total_hits as f64 / total_reads as f64
  } else {
    0.0
  };

  // Measure memory after all operations / 所有操作完成后测量内存
  let alloc_after = get_thread_allocated();
  let dealloc_after = get_thread_deallocated();

  // Calculate total memory used (cache + data) / 计算总内存使用（缓存+数据）
  let mem_before = alloc_before.saturating_sub(dealloc_before);
  let mem_after = alloc_after.saturating_sub(dealloc_after);
  let total_mem = mem_after.saturating_sub(mem_before);

  let actual_mem_mb = total_mem as f64 / (1024.0 * 1024.0);

  println!(
    "{name}: hit_rate={:.2}%, capacity={capacity}, budget={:.1}MB, actual={:.1}MB",
    hit_rate * 100.0,
    mem_budget as f64 / (1024.0 * 1024.0),
    actual_mem_mb,
  );

  // Measure actual throughput / 测量实际吞吐量
  let mut read_idx = 0;
  let mut write_idx = 0;
  let mut delete_idx = 0;
  let mut miss_idx = 0;
  let mut rng_perf = fastrand::Rng::with_seed(SEED + 2000);
  let perf_ops_count = 100_000;
  let start = std::time::Instant::now();

  for _ in 0..perf_ops_count {
    let op = rng_perf.u8(0..100);
    if op < config.read_ratio {
      if rng_perf.u8(0..100) < config.real_miss_ratio {
        if miss_idx < perf_ops.miss_indices.len() {
          let miss_data_idx = perf_ops.miss_indices[miss_idx];
          let key = miss_data_idx.to_le_bytes();
          black_box(cache.get(&key));
          miss_idx += 1;
        }
      } else {
        if read_idx < perf_ops.read_indices.len() {
          let data_idx = perf_ops.read_indices[read_idx];
          let key = data_idx.to_le_bytes();
          if !black_box(cache.get(&key)) {
            // Cache miss: refill from data source / 缓存miss：从数据源重新填充
            let (_, v) = &data[data_idx];
            cache.set(&key, v);
          }
          read_idx += 1;
        }
      }
    } else if op < config.read_ratio + config.write_ratio {
      if write_idx < perf_ops.write_indices.len() {
        let data_idx = perf_ops.write_indices[write_idx];
        let key = data_idx.to_le_bytes();
        let (_, v) = &data[data_idx];
        cache.set(&key, v);
        write_idx += 1;
      }
    } else {
      if delete_idx < perf_ops.delete_indices.len() {
        let data_idx = perf_ops.delete_indices[delete_idx];
        let key = data_idx.to_le_bytes();
        cache.del(&key);
        delete_idx += 1;
      }
    }
  }

  let elapsed = start.elapsed();
  let ops_per_second = perf_ops_count as f64 / elapsed.as_secs_f64();

  // Now run performance benchmark / 现在运行性能测试
  group.bench_function(BenchmarkId::new("mixed_workload", name), |b| {
    b.iter(|| {
      let mut read_idx = 0;
      let mut write_idx = 0;
      let mut delete_idx = 0;
      let mut miss_idx = 0;
      let mut rng = fastrand::Rng::with_seed(SEED + 1000);

      // Run 10000 operations per iteration / 每次迭代运行 10000 次操作
      for _ in 0..10_000 {
        let op = rng.u8(0..100);

        if op < config.read_ratio {
          if rng.u8(0..100) < config.real_miss_ratio {
            if miss_idx < bench_ops.miss_indices.len() {
              let miss_data_idx = bench_ops.miss_indices[miss_idx];
              let key = miss_data_idx.to_le_bytes();
              black_box(cache.get(&key));
              miss_idx += 1;
            }
          } else {
            if read_idx < bench_ops.read_indices.len() {
              let data_idx = bench_ops.read_indices[read_idx];
              let key = data_idx.to_le_bytes();
              if !black_box(cache.get(&key)) {
                // Cache miss: refill from data source / 缓存miss：从数据源重新填充
                let (_, v) = &data[data_idx];
                cache.set(&key, v);
              }
              read_idx += 1;
            }
          }
        } else if op < config.read_ratio + config.write_ratio {
          if write_idx < bench_ops.write_indices.len() {
            let data_idx = bench_ops.write_indices[write_idx];
            let key = data_idx.to_le_bytes();
            let (_, v) = &data[data_idx];
            cache.set(&key, v);
            write_idx += 1;
          }
        } else {
          if delete_idx < bench_ops.delete_indices.len() {
            let data_idx = bench_ops.delete_indices[delete_idx];
            let key = data_idx.to_le_bytes();
            cache.del(&key);
            delete_idx += 1;
          }
        }
      }
    });
  });

  group.finish();

  // Calculate effective ops considering miss latency / 计算考虑 miss 延迟的有效 ops
  let effective_ops = calc_effective_ops(ops_per_second, hit_rate, miss_latency_ns);

  results.push(BenchResult {
    lib: name.to_string(),
    hit_rate,
    ops_per_second,
    effective_ops,
    memory_mb: total_mem as f64 / (1024.0 * 1024.0),
    capacity,
    scale_factor,
  });
}

fn criterion_benchmark(c: &mut Criterion) {
  println!("\n=== Generating Test Data ===\n");

  let config = BenchConfig {
    mem_budget: 64 * 1024 * 1024, // 64MB - target budget for all libraries
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
    num_keys: 20_000, // Reduced for better cache hit rates
    zipf_s: config.zipf_s,
    seed: SEED,
  };

  let mut generator = ZipfDataGenerator::new(zipf_config);
  let data = generator.generate_all();

  let total_size = ZipfDataGenerator::total_size(&data);
  let avg_size = ZipfDataGenerator::avg_size(&data);
  let (min_size, max_size) = ZipfDataGenerator::size_range(&data);

  println!(
    "Generated {len} items, {total_mb:.2}MB total",
    len = data.len(),
    total_mb = total_size as f64 / (1024.0 * 1024.0)
  );
  println!("Avg size: {avg_size}B, Range: {min_size}B - {max_size}B");
  println!("Memory budget: {target_mem_mb}MB\n");

  // Estimate hot key average size / 估算热点键平均大小
  let hot_avg_size = estimate_hot_avg_size(&data, &mut generator);
  println!("Hot key avg size: {hot_avg_size}B\n");

  // Load previous results for automatic capacity calibration / 加载历史结果用于自动容量校准
  let previous_bench = load_previous_results();
  let previous_results = previous_bench.as_ref().map(|b| b.results.as_slice());

  if previous_results.is_some() {
    println!("Loaded previous results for automatic capacity calibration");
  } else {
    println!("No previous results found, using default capacities");
  }

  // Calculate size distribution / 计算大小分布
  let mut size_buckets = [(0usize, 0u64); 5];
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
      let labels = ["<100B", "100B-1KB", "1-10KB", "10-100KB", ">=100KB"];
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

  // Create benchmark parameters / 创建评测参数
  let params = BenchParams {
    mem_budget,
    avg_item_size: avg_size,
    previous_results,
    config: &config,
    miss_latency_ns,
  };

  // Benchmark each implementation / 评测每个实现
  #[cfg(feature = "bench-size-lru")]
  bench_cache::<adapter_size_lru::SizeLruAdapter>(c, &data, &params, &mut generator, &mut results);

  #[cfg(feature = "bench-moka")]
  bench_cache::<adapter_moka::MokaAdapter>(c, &data, &params, &mut generator, &mut results);

  #[cfg(feature = "bench-mini-moka")]
  bench_cache::<adapter_mini_moka::MiniMokaAdapter>(
    c,
    &data,
    &params,
    &mut generator,
    &mut results,
  );

  #[cfg(feature = "bench-clru")]
  bench_cache::<adapter_clru::ClruAdapter>(c, &data, &params, &mut generator, &mut results);

  #[cfg(feature = "bench-lru")]
  bench_cache::<adapter_lru::LruAdapter>(c, &data, &params, &mut generator, &mut results);

  #[cfg(feature = "bench-hashlink")]
  bench_cache::<adapter_hashlink::HashlinkAdapter>(c, &data, &params, &mut generator, &mut results);

  #[cfg(feature = "bench-schnellru")]
  bench_cache::<adapter_schnellru::SchnellruAdapter>(
    c,
    &data,
    &params,
    &mut generator,
    &mut results,
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
