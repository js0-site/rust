// Common benchmark utilities / 通用评测工具
// LruBench trait and shared data structures

use std::{path::Path, sync::OnceLock};

use rapidhash::RapidHashMap as HashMap;

use serde::{Deserialize, Serialize};

/// Cached ratio config / 缓存的比例配置
#[allow(dead_code)]
static RATIO_CONFIG_CACHE: OnceLock<std::sync::Mutex<RatioConfig>> = OnceLock::new();

/// Ratio calibration data / 比例校准数据
#[allow(dead_code)]
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct RatioConfig {
  pub target_mem_mb: u64,
  pub ratios: HashMap<String, RatioEntry>,
}

/// Single measurement record / 单次测量记录
#[allow(dead_code)]
#[derive(Serialize, Deserialize, Clone)]
pub struct Measurement {
  pub ratio: f64,
  pub mem_mb: f64,
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct RatioEntry {
  /// Historical measurements for averaging / 历史测量数据用于平均
  pub history: Vec<Measurement>,
}

/// Ratio config file path / 比例配置文件路径
#[allow(dead_code)]
pub const RATIO_JSON: &str = "benches/ratio.json";

/// Load ratio config from JSON, returns default if not found
/// 从 JSON 加载比例配置，找不到则返回默认值
#[allow(dead_code)]
pub fn load_ratio_config() -> RatioConfig {
  let cache = RATIO_CONFIG_CACHE.get_or_init(|| {
    let path = Path::new(RATIO_JSON);
    let config = if path.exists() {
      match std::fs::read_to_string(path) {
        Ok(json) => sonic_rs::from_str(&json).unwrap_or_default(),
        Err(_) => RatioConfig::default(),
      }
    } else {
      RatioConfig {
        target_mem_mb: 16,
        ratios: HashMap::new(),
      }
    };
    std::sync::Mutex::new(config)
  });

  cache.lock().unwrap().clone()
}

/// Get calibrated capacity for a library / 获取库的校准容量
/// Uses latest calculated ratio directly (already optimal)
/// 直接使用最新计算的 ratio（已是最优值）
#[allow(dead_code)]
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

/// LRU cache benchmark trait / LRU 缓存评测 trait
pub trait LruBench {
  /// Library name / 库名称
  const NAME: &'static str;

  /// Create cache with fixed memory budget (bytes)
  /// 使用固定内存预算（字节）创建缓存
  fn new(mem_budget: usize, target_mem_mb: u64) -> Self
  where
    Self: Sized;

  /// Set key-value pair / 设置键值对
  fn set(&mut self, key: &[u8], val: &[u8]);

  /// Get value, returns true if hit / 获取值，命中返回 true
  fn get(&mut self, key: &[u8]) -> bool;

  /// Delete key, returns true if existed / 删除键，存在返回 true
  fn del(&mut self, key: &[u8]) -> bool;
}

/// Benchmark configuration / 评测配置
#[derive(Serialize, Deserialize, Clone)]
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
