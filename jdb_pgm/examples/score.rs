use std::time::Instant;
use clap::Parser;
use jdb_pgm::pc::{
  Pc,
  types::{ExPenalty, PcConf},
};
use rand::prelude::*;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
  /// Epsilon for PGM
  #[arg(short, long, default_value_t = 8)]
  epsilon: usize,

  /// Exception Penalty for PFOR
  #[arg(long, default_value_t = 1)]
  ex_penalty: u8,

  /// Data size
  #[arg(short, long, default_value_t = 1_000_000)]
  n: usize,
}

const SEED: u64 = 12345;

fn measure_vec(data: &[u64], n_queries: usize) -> (f64, f64, f64) { // (size_mb, get_mops, p99_ns)
    let n = data.len();
    let boxed: Box<[u64]> = data.into();
    let size_mb = (boxed.len() * 8) as f64 / 1e6;

    let mut rng = StdRng::seed_from_u64(SEED);
    // Throughput
    let t = Instant::now();
    let mut chk = 0usize;
    for _ in 0..n_queries {
        let idx = rng.random_range(0..n);
        let val = unsafe { *boxed.get_unchecked(idx) };
        chk ^= val as usize;
    }
    std::hint::black_box(chk);
    let get_mops = (n_queries as f64 / 1e6) / t.elapsed().as_secs_f64();

    // Latency P99
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
    latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p99 = latencies[(latencies.len() as f64 * 0.99) as usize];

    (size_mb, get_mops, p99)
}

fn measure_pc(data: &[u64], conf: PcConf, n_queries: usize) -> (f64, f64, f64) {
    let n = data.len();
    let pc = Pc::new_with_conf(data, conf);
    let size_mb = pc.size_in_bytes() as f64 / 1e6;

    let mut rng = StdRng::seed_from_u64(SEED);
    // Throughput
    let t = Instant::now();
    let mut chk = 0u64;
    for _ in 0..n_queries {
        let idx = rng.random_range(0..n);
        chk ^= pc.get(idx).unwrap_or(0);
    }
    std::hint::black_box(chk);
    let get_mops = (n_queries as f64 / 1e6) / t.elapsed().as_secs_f64();

    // Latency P99
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
    latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p99 = latencies[(latencies.len() as f64 * 0.99) as usize];

    (size_mb, get_mops, p99)
}

fn main() {
  let args = Args::parse();
  let n = args.n;
  let n_queries = 100_000;

  // 1. Generate Data
  let mut rng = StdRng::seed_from_u64(42);
  // Zipf-like or just Uniform? The tuned params depend on distribution.
  // Using Uniform sparse for now as per original.
  let dist = rand_distr::Uniform::new(0, n as u64 * 10).unwrap();
  let mut data: Vec<u64> = (0..n).map(|_| dist.sample(&mut rng)).collect();
  data.sort_unstable();
  // Ensure unique for PGM if desired, but PGM handles duplicates.
  
  // 2. Baseline
  let (base_size, base_mops, base_p99) = measure_vec(&data, n_queries);

  // 3. Candidate
  let conf = PcConf {
    epsilon: args.epsilon,
    ex_penalty: ExPenalty::new(args.ex_penalty),
  };
  let (pc_size, pc_mops, pc_p99) = measure_pc(&data, conf, n_queries);

  // 4. Check Constraints
  // DRAM <= 30% or Reduction >= 70%
  let size_ratio = pc_size / base_size;
  let dram_pass = size_ratio <= 0.30;

  // Throughput >= 95%
  let tpt_ratio = pc_mops / base_mops;
  let tpt_pass = tpt_ratio >= 0.95;

  // Latency P99 increase <= 10% (Ratio <= 1.10)
  let lat_ratio = pc_p99 / base_p99;
  let lat_pass = lat_ratio <= 1.10;

  // 5. Scoring
  // Metric: Efficiency = Throughput / SizeRatio
  // This naturally rewards high throughput and high compression.
  let raw_score = pc_mops / size_ratio;
  
  let score;
  if dram_pass && tpt_pass && lat_pass {
      score = raw_score;
  } else {
      // User request: Failures should have a score to provide gradients, but penalized significantly.
      score = raw_score / 100.0;
  }

  println!("{:.4}", score);
  eprintln!(
    "Result: N={} Eps={} Pen={}\n  Base: Size={:.2}MB Mops={:.2} P99={:.0}ns\n  Pc:   Size={:.2}MB Mops={:.2} P99={:.0}ns\n  Ratios: Size={:.2} (Goal<=0.30) Tpt={:.2} (Goal>=0.95) Lat={:.2} (Goal<=1.10)\n  Pass: {:?} Score={:.4}",
    n, args.epsilon, args.ex_penalty, 
    base_size, base_mops, base_p99,
    pc_size, pc_mops, pc_p99,
    size_ratio, tpt_ratio, lat_ratio,
    dram_pass && tpt_pass && lat_pass, score
  );
}
