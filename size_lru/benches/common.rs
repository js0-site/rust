// Common benchmark utilities / 通用评测工具
// LruBench trait and shared data structures

use std::{collections::HashMap, path::Path, sync::OnceLock};

use serde::{Deserialize, Serialize};

/// Ratio calibration data / 比例校准数据
#[derive(Serialize, Deserialize, Default)]
pub struct RatioConfig {
  pub target_mem_mb: u64,
  pub ratios: HashMap<String, RatioEntry>,
}

/// Single measurement record / 单次测量记录
#[derive(Serialize, Deserialize, Clone)]
pub struct Measurement {
  pub ratio: f64,
  pub mem_mb: f64,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct RatioEntry {
  /// Historical measurements for averaging / 历史测量数据用于平均
  pub history: Vec<Measurement>,
}

/// Ratio config file path / 比例配置文件路径
pub const RATIO_JSON: &str = "benches/ratio.json";

/// Load ratio config from JSON, returns default if not found
/// 从 JSON 加载比例配置，找不到则返回默认值
pub fn load_ratio_config() -> RatioConfig {
  let path = Path::new(RATIO_JSON);
  if !path.exists() {
    return RatioConfig {
      target_mem_mb: 16,
      ratios: HashMap::new(),
    };
  }
  match std::fs::read_to_string(path) {
    Ok(json) => sonic_rs::from_str(&json).unwrap_or_default(),
    Err(_) => RatioConfig::default(),
  }
}

/// Save ratio config to JSON / 保存比例配置到 JSON
pub fn save_ratio_config(cfg: &RatioConfig) {
  let json = sonic_rs::to_string_pretty(cfg).expect("serialize ratio.json");
  std::fs::write(RATIO_JSON, json).expect("write ratio.json");
}

/// Max history entries per library / 每个库最大历史记录数
const MAX_HISTORY: usize = 5;

/// Get calibrated capacity for a library / 获取库的校准容量
/// Uses latest calculated ratio directly (already optimal)
/// 直接使用最新计算的 ratio（已是最优值）
pub fn calibrated_cap(lib: &str, mem_budget: usize, _target_mem_mb: u64) -> usize {
  let cfg = load_ratio_config();

  // Use latest ratio directly, default 1.0
  // 直接使用最新 ratio，默认 1.0
  let ratio = cfg
    .ratios
    .get(lib)
    .and_then(|e| e.history.last())
    .map(|m| m.ratio)
    .unwrap_or(1.0);

  let cap = (mem_budget as f64 * ratio) as usize;
  cap.max(1)
}

/// Update ratio entry with new measurement / 用新测量更新比例条目
/// Keeps only recent MAX_HISTORY entries for observation
/// 只保留最近 MAX_HISTORY 条用于观察
pub fn update_ratio_entry(entry: &mut RatioEntry, new_ratio: f64, mem_mb: f64) {
  entry.history.push(Measurement {
    ratio: new_ratio,
    mem_mb,
  });

  // Keep only recent entries / 只保留最近的记录
  if entry.history.len() > MAX_HISTORY {
    entry.history.remove(0);
  }
}

/// LRU cache benchmark trait / LRU 缓存评测 trait
pub trait LruBench {
  /// Create cache with fixed memory budget (bytes)
  /// 使用固定内存预算（字节）创建缓存
  fn new(mem_budget: usize, target_mem_mb: u64) -> Self
  where
    Self: Sized;

  /// Library name / 库名称
  fn name(&self) -> &'static str;

  /// Set key-value pair / 设置键值对
  fn set(&mut self, key: &[u8], val: &[u8]);

  /// Get value, returns true if hit / 获取值，命中返回 true
  fn get(&mut self, key: &[u8]) -> bool;

  /// Delete key, returns true if existed / 删除键，存在返回 true
  fn del(&mut self, key: &[u8]) -> bool;
}

/// Benchmark configuration / 评测配置
#[derive(Serialize, Clone)]
pub struct BenchConfig {
  /// Memory budget in bytes / 内存预算（字节）
  pub mem_budget: usize,
  /// Read operation ratio (%) / 读操作比例
  pub read_ratio: u8,
  /// Write operation ratio (%) / 写操作比例
  pub write_ratio: u8,
  /// Delete operation ratio (%) / 删操作比例
  pub delete_ratio: u8,
  /// Real miss ratio - requests for non-existent keys (%) / 真实miss比例
  pub real_miss_ratio: u8,
  /// Zipf exponent / Zipf 指数
  pub zipf_s: f64,
  /// Operations per benchmark loop / 每轮操作数
  pub ops_per_loop: usize,
  /// Number of benchmark loops / 评测轮数
  pub loops: usize,
}

impl Default for BenchConfig {
  fn default() -> Self {
    Self {
      mem_budget: 64 * 1024 * 1024, // 64MB
      read_ratio: 90,
      write_ratio: 9,
      delete_ratio: 1,
      real_miss_ratio: 5,
      zipf_s: 1.0,               // s=1.0 → 20% keys get ~84% accesses (80/20 rule)
      ops_per_loop: 120_000_000, // 120M ops per loop
      loops: 3,
    }
  }
}

/// Size distribution bucket / 大小分布桶
#[derive(Serialize, Clone)]
pub struct SizeBucket {
  /// Bucket label (e.g., "<1KB", "1-4KB") / 桶标签
  pub label: String,
  /// Number of items in this bucket / 此桶中的条目数
  pub count: usize,
  /// Percentage of total items / 占总条目的百分比
  pub percent: f64,
  /// Total size of items in this bucket (bytes) / 此桶中条目的总大小（字节）
  pub total_size_bytes: u64,
  /// Percentage of total size / 占总大小的百分比
  pub size_percent: f64,
}

/// Dataset statistics / 数据集统计
#[derive(Serialize, Clone)]
pub struct DatasetStats {
  /// Total data size in bytes / 数据总大小（字节）
  pub total_size_bytes: u64,
  /// Number of unique items / 唯一条目数
  pub item_count: usize,
  /// Average item size in bytes / 平均条目大小（字节）
  pub avg_item_size: usize,
  /// Min item size in bytes / 最小条目大小（字节）
  pub min_item_size: usize,
  /// Max item size in bytes / 最大条目大小（字节）
  pub max_item_size: usize,
  /// Memory budget for all caches (bytes) / 所有缓存的内存预算（字节）
  pub mem_budget: u64,
  /// Size distribution / 大小分布
  pub size_distribution: Vec<SizeBucket>,
}

/// Single benchmark result / 单次评测结果
#[derive(Serialize, Clone)]
pub struct BenchResult {
  pub lib: String,
  pub hit_rate: f64,
  pub ops_per_second: f64,
  pub effective_ops: f64,
  pub memory_kb: f64,
}

/// Full benchmark output / 完整评测输出
#[derive(Serialize)]
pub struct BenchOutput {
  pub config: BenchConfig,
  pub miss_latency_ns: u64,
  pub miss_latency_method: String,
  pub stats: DatasetStats,
  pub results: Vec<BenchResult>,
}

/// Data directory / 数据目录
pub const DATA_DIR: &str = "../jdb_bench_data/data";

/// JSON output path / JSON 输出路径
pub const JSON_PATH: &str = "bench.json";

/// Global miss latency cache / 全局 miss 延迟缓存
static MISS_LATENCY: OnceLock<u64> = OnceLock::new();

/// Return 2025 NVMe 4K random read average latency / 返回2025年NVMe 4K随机读平均延迟
pub fn estimate_miss_latency() -> u64 {
  *MISS_LATENCY.get_or_init(|| {
    // 2025 DapuStor Xlenstor X5900 Series PCIe 5.0 enterprise NVMe 4K random read latency: 18 microseconds
    // Source: FMS 2025 announcement - X5900 Series engineered for latency-sensitive workloads
    // Features industry-leading 4KB read/write latencies as low as 18/5 μs with ultra-high endurance of 120 DWPD
    // Optimized for AI inference, real-time analytics, and high-frequency trading applications
    // 2025年DapuStor Xlenstor X5900系列PCIe 5.0企业级NVMe 4K随机读延迟：18微秒
    // 来源：FMS 2025发布 - X5900系列专为延迟敏感工作负载设计
    // 具备业界领先的4KB读/写延迟，低至18/5微秒，超高耐用性120 DWPD
    // 针对AI推理、实时分析和高频交易应用优化
    18_000 // 18 microseconds in nanoseconds
  })
}

/// Miss latency estimation method description / miss 延迟估算方法描述
pub fn miss_latency_method() -> &'static str {
  "DapuStor X5900 PCIe 5.0 NVMe (18µs)"
}

/// Calculate effective OPS considering miss latency / 计算考虑 miss 延迟的有效 OPS
#[inline]
pub fn calc_effective_ops(ops_per_second: f64, hit_rate: f64, miss_latency_ns: u64) -> f64 {
  let hit_time_ns = 1e9 / ops_per_second;
  let miss_rate = 1.0 - hit_rate;
  let avg_time_ns = hit_time_ns + miss_rate * miss_latency_ns as f64;
  1e9 / avg_time_ns
}
