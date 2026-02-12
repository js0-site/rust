use std::time::Instant;

use clap::Parser;
use jdb_pgm::pc::Pc;
use rand::{
  SeedableRng,
  distr::{Distribution, Uniform},
  prelude::StdRng,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
  #[arg(short, long, default_value_t = 1_000_000)]
  n: usize,

  #[arg(short, long, default_value = "uniform")]
  dist: String,

  #[arg(short, long, default_value_t = 64)]
  epsilon: usize,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let args = Args::parse();
  println!(
    "Generating {} items with {} distribution...",
    args.n, args.dist
  );

  let mut rng = StdRng::seed_from_u64(42);
  let data: Vec<u64> = if args.dist == "zipf" {
    // Zipf-like simulation (basic)
    let dist = Uniform::new(0, args.n as u64 * 10).unwrap();
    let mut d: Vec<u64> = (0..args.n).map(|_| dist.sample(&mut rng)).collect();
    d.sort();
    d.dedup();
    d
  } else {
    // Uniform unique keys
    (0..args.n as u64).collect()
  };

  println!("Building Pc index (Epsilon={})...", args.epsilon);
  let start = Instant::now();
  let pc = Pc::new(&data, args.epsilon);
  println!("Build time: {:?}", start.elapsed());

  pc.print_stats();

  Ok(())
}
