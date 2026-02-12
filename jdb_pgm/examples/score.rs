use std::time::Instant;

use clap::Parser;
use jdb_pgm::pc::{
  Pc,
  types::{ExPenalty, PcConf},
};
use rand::prelude::*;
use rand_distr::Distribution;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
  /// Epsilon for PGM
  /// PGM 的 Epsilon 参数
  #[arg(short, long, default_value_t = 8)]
  epsilon: usize,

  /// Exception Penalty for PFOR
  /// PFOR 的异常惩罚因子
  #[arg(long, default_value_t = 1)]
  ex_penalty: u8,

  /// Data size (default 1000 MiB = 131,072,000 u64s)
  /// 数据大小（默认 1000 MiB = 131,072,000 u64s）
  #[arg(short, long, default_value_t = 131_072_000)]
  n: usize,
}

const SEED: u64 = 12345;

fn measure_vec(data: &[u64], n_queries: usize) -> (f64, f64, f64) {
  // (size_mb, get_mops, p99_ns)
  let n = data.len();
  // Simulate standard slice access overhead
  // 模拟标准切片访问开销
  let boxed: Box<[u64]> = data.into();
  let size_mb = (boxed.len() * 8) as f64 / 1024.0 / 1024.0;

  let mut rng = StdRng::seed_from_u64(SEED);
  // Throughput
  // Pre-generate indices to exclude RNG cost
  // 预生成索引以排除 RNG 开销
  let indices: Vec<usize> = (0..n_queries).map(|_| rng.random_range(0..n)).collect();

  let t = Instant::now();
  let mut chk = 0usize;
  for &idx in &indices {
    // SAFETY: indices are within bounds
    // 安全性：索引在边界内
    let val = unsafe { *boxed.get_unchecked(idx) };
    chk ^= val as usize;
  }
  std::hint::black_box(chk);
  let get_mops = (n_queries as f64 / 1e6) / t.elapsed().as_secs_f64();

  // Latency P99
  // P99 延迟
  let mut rng = StdRng::seed_from_u64(SEED + 1);
  let mut latencies = Vec::with_capacity(n_queries);
  for _ in 0..n_queries {
    let idx = rng.random_range(0..n);
    let start = Instant::now();
    let val = unsafe { *boxed.get_unchecked(idx) };
    let d = start.elapsed();
    std::hint::black_box(val);
    latencies.push(d.as_nanos() as f64);
  }
  // No unwrap needed for f64 sorting if no NaNs
  // 如果没有 NaNs，f64 排序不需要 unwrap
  latencies.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
  let p99 = latencies[(latencies.len() as f64 * 0.99) as usize];

  (size_mb, get_mops, p99)
}

fn measure_pc(data: &[u64], conf: PcConf, n_queries: usize) -> (f64, f64, f64) {
  let n = data.len();
  let pc = Pc::new_with_conf(data, conf);
  let size_mb = pc.size_in_bytes() as f64 / 1024.0 / 1024.0;

  let mut rng = StdRng::seed_from_u64(SEED);
  // Throughput
  // Pre-generate indices
  // 预生成索引
  let indices: Vec<usize> = (0..n_queries).map(|_| rng.random_range(0..n)).collect();

  let t = Instant::now();
  let mut chk = 0u64;
  for &idx in &indices {
    // SAFETY: indices are pre-generated within bounds
    // 安全性：索引是预生成的，在范围内
    chk ^= unsafe { pc.get_unchecked(idx) };
  }
  std::hint::black_box(chk);
  let get_mops = (n_queries as f64 / 1e6) / t.elapsed().as_secs_f64();

  // Latency P99
  // P99 延迟
  let mut rng = StdRng::seed_from_u64(SEED + 1);
  let mut latencies = Vec::with_capacity(n_queries);
  for _ in 0..n_queries {
    let idx = rng.random_range(0..n);
    let start = Instant::now();
    let val = pc.get(idx).unwrap_or(0);
    let d = start.elapsed();
    std::hint::black_box(val);
    latencies.push(d.as_nanos() as f64);
  }
  latencies.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
  let p99 = latencies[(latencies.len() as f64 * 0.99) as usize];

  (size_mb, get_mops, p99)
}

fn main() {
  let args = Args::parse();
  let n = args.n;
  let n_queries = 100_000;

  // 1. Generate Data (Simulate FLT / Disk Data)
  // Use LogNormal random floats, then reinterpret as u64 to simulate floating point keys.
  // This creates a distribution similar to the 'f_books' or synthetic float workloads.
  // 1. 生成数据（模拟 FLT / 磁盘数据）
  // 使用对数正态随机浮点数，然后重新解释为 u64 以模拟浮点键。
  // 这创建了一个类似于 'f_books' 或合成浮点工作负载的分布。
  let mut rng = StdRng::seed_from_u64(42);
  let dist = rand_distr::LogNormal::new(0.0, 1.0).unwrap();

  let mut data: Vec<u64> = (0..n)
    .map(|_| {
      let f: f64 = dist.sample(&mut rng);
      f.to_bits()
    })
    .collect();

  // Sort to simulate sorted index build input
  // 排序以模拟有序索引构建输入
  data.sort_unstable();

  // Dedup is optional for PGM but good for meaningful 1GB size test
  // Dedup 对于 PGM 是可选的，但对于有意义的 1GB 大小测试很有用
  data.dedup();
  if data.len() < n {
    eprintln!("Warning: Dedup reduced size from {} to {}", n, data.len());
  }
  // Ensure unique for PGM if desired, but PGM handles duplicates.
  // 如果需要，确保 PGM 的唯一性，但 PGM 可以处理重复项。

  // 2. Baseline
  // 2. 基准测试
  let (base_size, base_mops, base_p99) = measure_vec(&data, n_queries);

  // 3. Candidate
  // 3. 候选测试
  let conf = PcConf {
    epsilon: args.epsilon,
    ex_penalty: ExPenalty::new(args.ex_penalty),
  };
  let (pc_size, pc_mops, pc_p99) = measure_pc(&data, conf, n_queries);

  // 4. Check Constraints
  // DRAM <= 30% or Reduction >= 70%
  // 4. 检查约束
  // DRAM <= 30% 或缩减 >= 70%
  let size_ratio = pc_size / base_size;
  let dram_pass = size_ratio <= 0.30;

  // Throughput >= 95%
  // 吞吐量 >= 95%
  let tpt_ratio = pc_mops / base_mops;
  let tpt_pass = tpt_ratio >= 0.95;

  // Latency P99 increase <= 10% (Ratio <= 1.10)
  // 延迟 P99 增加 <= 10% (比率 <= 1.10)
  let lat_ratio = pc_p99 / base_p99;
  let lat_pass = lat_ratio <= 1.10;

  // 5. Scoring
  // Metric: Efficiency = Throughput / SizeRatio
  // This naturally rewards high throughput and high compression.
  // 5. 评分
  // 指标：效率 = 吞吐量 / 尺寸比率
  // 这自然奖励高吞吐量和高压缩率。
  let raw_score = pc_mops / size_ratio;

  let score = if dram_pass && tpt_pass && lat_pass {
    raw_score
  } else {
    // User request: Failures should have a score to provide gradients, but penalized significantly.
    // 用户请求：失败应该有分数以提供梯度，但会受到显着惩罚。
    raw_score / 100.0
  };

  println!("{:.4}", score);
  eprintln!(
    "Result: N={} Eps={} Pen={}\n  Base: Size={:.2}MB Mops={:.2} P99={:.0}ns\n  Pc:   Size={:.2}MB Mops={:.2} P99={:.0}ns\n  Ratios: Size={:.2} (Goal<=0.30) Tpt={:.2} (Goal>=0.95) Lat={:.2} (Goal<=1.10)\n  Pass: {:?} Score={:.4}",
    n,
    args.epsilon,
    args.ex_penalty,
    base_size,
    base_mops,
    base_p99,
    pc_size,
    pc_mops,
    pc_p99,
    size_ratio,
    tpt_ratio,
    lat_ratio,
    dram_pass && tpt_pass && lat_pass,
    score
  );
}
