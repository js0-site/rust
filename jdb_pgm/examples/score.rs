use std::time::Instant;

use clap::Parser;
use jdb_pgm::pc::Pc;
use rand::prelude::*;
use rand_distr::Distribution;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
  /// Epsilon for PGM
  #[arg(short, long, default_value_t = 8)]
  epsilon: usize,

  /// Data size
  #[arg(short, long, default_value_t = 1_000_000)]
  n: usize,
}

fn main() {
  let args = Args::parse();

  // 1. Generate Data (Zipf)
  let n = args.n;
  let mut rng = StdRng::seed_from_u64(42);
  let dist = rand_distr::Uniform::new(0, n as u64 * 10).unwrap();
  let mut data: Vec<u64> = (0..n).map(|_| dist.sample(&mut rng)).collect();
  data.sort_unstable();
  data.dedup(); // PGM requires unique keys? PGM handles duplicates usually but Pc assumes keys.
  // Pc works on sorted u64. Duplicates allowed in logic but usually unique keys for Index.
  // Let's ensure strict monotonicity for simplicity, or just allow duplicates.
  // Pc standard assumes sorted.
  // If dedup reduces N drastically, we might want to fill.
  // Just sort is enough.

  // 2. Build
  let start = Instant::now();
  let pc = Pc::new(&data, args.epsilon);
  let build_dur = start.elapsed();
  let build_mops = (data.len() as f64 / 1e6) / build_dur.as_secs_f64();

  // 3. Size
  let size_bytes = pc.size_in_bytes();
  let size_mb = size_bytes as f64 / 1e6;

  // 4. Random Access
  // Query 100k random keys
  let n_queries = 200_000;
  // Use StdRng for deterministic queries
  let mut query_rng = StdRng::seed_from_u64(12345);
  let indices: Vec<usize> = (0..n_queries)
    .map(|_| query_rng.random_range(0..pc.len))
    .collect();

  let start = Instant::now();
  let mut checksum = 0;
  for &idx in &indices {
    checksum += pc.get(idx).expect("Should exist");
  }
  let get_dur = start.elapsed();
  let get_mops = (n_queries as f64 / 1e6) / get_dur.as_secs_f64();

  // Prevent optimization
  if checksum == 1 {
    println!("?");
  }

  // 5. Score
  // Formula: Prioritize Compression (Size) and Random Access.
  // Score = RandomMops / (SizeMB^2)
  // E.g.
  // Old: 100 Mops, 1.33 MB => 100 / 1.76 = 56.8
  // PFOR ideal: 90 Mops, 1.1 MB => 90 / 1.21 = 74.3
  // This strongly rewards compression.
  let score = get_mops / (size_mb * size_mb);

  println!("{}", score);
  eprintln!(
    "Details: N={} Size={:.2}MB Get={:.2}M/s Build={:.2}M/s Eps={} Score={:.2}",
    pc.len, size_mb, get_mops, build_mops, args.epsilon, score
  );
}
