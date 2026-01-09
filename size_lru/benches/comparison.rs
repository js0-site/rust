// LRU cache performance comparison benchmark
// LRU 缓存性能对比基准测试

use std::{io::Write, path::Path, time::Instant};

use jdb_bench_data::{Jemalloc, MemBaseline, SEED, WorkloadConfig, ZipfSampler, load_workload};

// Global allocator / 全局分配器
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

mod common;
use common::{
  BenchConfig, BenchOutput, BenchResult, DATA_DIR, JSON_PATH, LruBench, calc_effective_ops,
  estimate_miss_latency, load_ratio_config, miss_latency_method, save_ratio_config,
  update_ratio_entry,
};

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

/// Generate a "real miss" key / 生成真实 miss key
fn gen_miss_key(rng: &mut fastrand::Rng) -> Vec<u8> {
  let mut key = b"__MISS__".to_vec();
  key.extend_from_slice(&rng.u64(..).to_le_bytes());
  key
}

/// Run benchmark for a single cache implementation
/// 运行单个缓存实现的评测
fn run_bench<C: LruBench>(
  data: &[KvPair],
  mem_budget: usize,
  target_mem_mb: u64,
  miss_latency_ns: u64,
  config: &BenchConfig,
) -> BenchResult {
  let n = data.len();
  if n == 0 {
    let cache = C::new(mem_budget, target_mem_mb);
    return BenchResult {
      lib: cache.name().to_string(),
      hit_rate: 0.0,
      ops_per_second: 0.0,
      effective_ops: 0.0,
      memory_kb: 0.0,
    };
  }

  let sampler = ZipfSampler::new(n, config.zipf_s);
  let mem_before = MemBaseline::record();

  let mut cache = C::new(mem_budget, target_mem_mb);
  let name = cache.name();

  // Warmup once: fill cache with sequential data
  // 预热一次：顺序填充缓存
  for (k, v) in data {
    cache.set(k, v);
  }

  let mut total_hits = 0u64;
  let mut total_reads = 0u64;
  let mut total_ops = 0u64;
  let mut total_time_ns = 0u64;

  for loop_idx in 0..config.loops {
    let mut rng = fastrand::Rng::with_seed(SEED + loop_idx as u64);
    let mut hits = 0u64;
    let mut reads = 0u64;

    // Benchmark with realistic workload / 真实工作负载评测
    let start = Instant::now();
    for _ in 0..config.ops_per_loop {
      let op = rng.u8(0..100);

      if op < config.read_ratio {
        reads += 1;
        if rng.u8(0..100) < config.real_miss_ratio {
          let miss_key = gen_miss_key(&mut rng);
          cache.get(&miss_key);
        } else {
          let idx = sampler.sample(&mut rng);
          let (k, _) = &data[idx];
          if cache.get(k) {
            hits += 1;
          }
        }
      } else if op < config.read_ratio + config.write_ratio {
        let idx = sampler.sample(&mut rng);
        let (k, v) = &data[idx];
        cache.set(k, v);
      } else {
        let idx = sampler.sample(&mut rng);
        let (k, _) = &data[idx];
        cache.del(k);
      }
    }
    let elapsed = start.elapsed();

    total_hits += hits;
    total_reads += reads;
    total_ops += config.ops_per_loop as u64;
    total_time_ns += elapsed.as_nanos() as u64;
  }

  // Measure memory after all loops
  // 所有轮次结束后测量内存
  let final_mem_kb = mem_before.db_mem() as f64 / 1024.0;

  let hit_rate = if total_reads > 0 {
    total_hits as f64 / total_reads as f64
  } else {
    0.0
  };
  let ops_per_second = total_ops as f64 / (total_time_ns as f64 / 1e9);
  let effective_ops = calc_effective_ops(ops_per_second, hit_rate, miss_latency_ns);

  BenchResult {
    lib: name.to_string(),
    hit_rate,
    ops_per_second,
    effective_ops,
    memory_kb: final_mem_kb,
  }
}

/// Run all benchmarks and update ratio.json
/// 运行所有评测并更新 ratio.json
fn run_all_benchmarks(
  data: &[KvPair],
  mem_budget: usize,
  target_mem_mb: u64,
  miss_latency_ns: u64,
  config: &BenchConfig,
) -> Vec<BenchResult> {
  let mut results = Vec::new();
  let mut ratio_cfg = load_ratio_config();
  ratio_cfg.target_mem_mb = target_mem_mb;

  macro_rules! bench {
    ($feature:literal, $adapter:ty) => {
      #[cfg(feature = $feature)]
      {
        let r = run_bench::<$adapter>(data, mem_budget, target_mem_mb, miss_latency_ns, config);
        println!(
          "  {}: hit={:.1}%, mem={:.0}KB, eff_ops={:.2}M/s",
          r.lib,
          r.hit_rate * 100.0,
          r.memory_kb,
          r.effective_ops / 1e6
        );

        // Update ratio: optimal = old * target / actual
        // 更新比例：最优 = 旧值 * 目标 / 实际
        let actual_mem_mb = r.memory_kb / 1024.0;
        let old_ratio = ratio_cfg
          .ratios
          .get(&r.lib)
          .and_then(|e| e.history.last())
          .map(|m| m.ratio)
          .unwrap_or(1.0);
        let new_ratio = old_ratio * target_mem_mb as f64 / actual_mem_mb;

        let entry = ratio_cfg.ratios.entry(r.lib.clone()).or_default();
        update_ratio_entry(entry, new_ratio, actual_mem_mb);

        results.push(r);
      }
    };
  }

  bench!("bench-size-lru", adapter_size_lru::SizeLruAdapter);
  bench!("bench-moka", adapter_moka::MokaAdapter);
  bench!("bench-mini-moka", adapter_mini_moka::MiniMokaAdapter);
  bench!("bench-clru", adapter_clru::ClruAdapter);
  bench!("bench-lru", adapter_lru::LruAdapter);
  bench!("bench-hashlink", adapter_hashlink::HashlinkAdapter);
  bench!("bench-schnellru", adapter_schnellru::SchnellruAdapter);

  // Save updated ratios / 保存更新后的比例
  save_ratio_config(&ratio_cfg);

  results
}

fn main() {
  let config = BenchConfig::default();
  let target_mem_mb = (config.mem_budget / 1024 / 1024) as u64;

  println!("Loading workload data...");

  // Use new workload API with Facebook USR/APP/VAR distribution
  // 使用新的工作负载 API，采用 Facebook USR/APP/VAR 分布
  let workload_config = WorkloadConfig::default()
    .with_zipf_s(config.zipf_s)
    .with_seed(SEED)
    .with_total_size(2000 * 1024 * 1024); // 2GB

  let workload = load_workload(Path::new(DATA_DIR), workload_config)
    .expect("Failed to load workload. Run: cd jdb_bench_data && ./init.sh");

  // Convert to KvPair format / 转换为 KvPair 格式
  let mixed_data: Vec<KvPair> = workload
    .data()
    .iter()
    .map(|(k, v)| (k.as_bytes().to_vec(), v.clone()))
    .collect();

  println!(
    "Workload: {} items, {}KB total, avg={}B",
    workload.len(),
    workload.total_size() / 1024,
    workload.avg_size()
  );

  println!("\nEstimating miss latency...");
  let miss_latency_ns = estimate_miss_latency();
  println!(
    "Miss latency: {miss_latency_ns}ns ({:.2}µs)",
    miss_latency_ns as f64 / 1000.0
  );

  let mem_budget = config.mem_budget;
  println!(
    "\nConfig: mem_budget={target_mem_mb}MB, read={:.0}%, write={:.0}%, delete={:.0}%, real_miss={:.0}%",
    config.read_ratio, config.write_ratio, config.delete_ratio, config.real_miss_ratio
  );

  println!("\nRunning benchmarks...");
  let results = run_all_benchmarks(
    &mixed_data,
    mem_budget,
    target_mem_mb,
    miss_latency_ns,
    &config,
  );

  // Use stats from workload / 使用工作负载的统计
  let stats = workload.stats().clone();

  // Create output / 创建输出
  let output = BenchOutput {
    config,
    miss_latency_ns,
    miss_latency_method: miss_latency_method().to_string(),
    stats: common::DatasetStats {
      total_size_bytes: stats.total_size_bytes,
      item_count: stats.item_count,
      avg_item_size: stats.avg_item_size,
      min_item_size: stats.min_item_size,
      max_item_size: stats.max_item_size,
      mem_budget: mem_budget as u64,
      size_distribution: stats
        .size_distribution
        .iter()
        .map(|b| common::SizeBucket {
          label: b.label.clone(),
          count: b.count,
          percent: b.percent,
          total_size_bytes: b.total_size_bytes,
          size_percent: b.size_percent,
        })
        .collect(),
    },
    results,
  };

  let json = sonic_rs::to_string_pretty(&output).expect("JSON serialize");
  let mut file = std::fs::File::create(JSON_PATH).expect("create bench.json");
  file.write_all(json.as_bytes()).expect("write bench.json");

  println!("\nResults written to {JSON_PATH}");
}
