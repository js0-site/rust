//! Common benchmark utilities and trait
//! 通用评测工具和 trait

use std::time::Instant;

use tikv_jemalloc_ctl::{epoch, stats};

/// Benchmark result for a single algorithm
/// 单个算法的评测结果
#[derive(Clone, Debug)]
pub struct BenchResult {
  pub name: String,
  pub data_size: usize,
  pub epsilon: Option<usize>,
  pub algorithm: String,
  pub mean_ns: f64,
  pub std_dev_ns: f64,
  pub median_ns: f64,
  pub throughput: f64,
  pub memory_bytes: usize,
}

/// Trait for benchmarkable index implementations
/// 可评测索引实现的 trait
pub trait Benchmarkable: Sized {
  /// Algorithm name
  /// 算法名称
  const NAME: &'static str;

  /// Build index from sorted data
  /// 从已排序数据构建索引
  fn build(data: &[u64], epsilon: Option<usize>) -> Self;

  /// Query a key, return position if found
  /// 查询键，找到则返回位置
  fn query(&self, data: &[u64], key: u64) -> Option<usize>;

  /// Generate benchmark name with optional epsilon
  /// 生成带可选 epsilon 的基准测试名称
  fn bench_name(eps: Option<usize>) -> String {
    match eps {
      Some(e) => format!("{}_{e}", Self::NAME),
      None => Self::NAME.to_string(),
    }
  }
}

/// Measure memory allocation using jemalloc
/// 使用 jemalloc 测量内存分配
pub fn measure_memory<F, T>(f: F) -> (T, usize)
where
  F: FnOnce() -> T,
{
  let _ = epoch::advance();
  let before = stats::allocated::read().unwrap_or(0);
  let result = f();
  let _ = epoch::advance();
  let after = stats::allocated::read().unwrap_or(0);
  (result, after.saturating_sub(before))
}

/// Calculate statistics from timing data
/// 从计时数据计算统计信息
pub fn calc_stats(times: &mut [f64]) -> (f64, f64, f64) {
  if times.is_empty() {
    return (0.0, 0.0, 0.0);
  }
  times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
  let n = times.len() as f64;
  let mean = times.iter().sum::<f64>() / n;
  let variance = times.iter().map(|&t| (t - mean).powi(2)).sum::<f64>() / n;
  (mean, variance.sqrt(), times[times.len() / 2])
}

/// Run benchmark for a given implementation
/// 运行给定实现的评测
pub fn run_bench<T: Benchmarkable>(
  data: &[u64],
  queries: &[u64],
  epsilon: Option<usize>,
) -> BenchResult {
  let (index, mem) = measure_memory(|| T::build(data, epsilon));

  let mut times: Vec<f64> = queries
    .iter()
    .map(|&q| {
      let start = Instant::now();
      let _ = index.query(data, q);
      start.elapsed().as_nanos() as f64
    })
    .collect();

  let (mean, std_dev, median) = calc_stats(&mut times);
  let algo = T::NAME.to_string();
  let name = match epsilon {
    Some(e) => format!("{}_eps_{e}/{}", algo, data.len()),
    None => format!("{}/{}", algo, data.len()),
  };

  BenchResult {
    name,
    data_size: data.len(),
    epsilon,
    algorithm: algo,
    mean_ns: mean,
    std_dev_ns: std_dev,
    median_ns: median,
    throughput: if mean > 0.0 { 1e9 / mean } else { 0.0 },
    memory_bytes: mem,
  }
}
