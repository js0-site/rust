use std::{hint::black_box, time::Instant};

use jdb_pgm::{Pc, pc::types::DEFAULT_EPSILON};
use rand::prelude::*;

use crate::library::{Library, Metrics, N_QUERIES, SEED};

pub struct PgmLib;
impl Library for PgmLib {
  const NAME: &'static str = "pc";

  fn measure(data: &[u64]) -> Metrics {
    let n = data.len();
    let orig = n * 8;

    // Build
    let t = Instant::now();
    let pc = Pc::new(data, DEFAULT_EPSILON);
    let encoded = pc.dump();
    let build_mops = n as f64 / 1e6 / t.elapsed().as_secs_f64();

    let total = encoded.len();

    // Validate load
    let loaded = Pc::load(&encoded).unwrap();
    assert_eq!(loaded.len, pc.len);
    // Note: get(0) might differ slightly due to float precision if data[0] is special,
    // but should be fine for u64.
    assert_eq!(loaded.get(0).unwrap(), pc.get(0).unwrap());

    // Random access
    // Pre-generate indices
    let mut rng = StdRng::seed_from_u64(SEED);
    let nq = N_QUERIES.min(n);
    let indices: Vec<usize> = (0..nq).map(|_| rng.random_range(0..n)).collect();

    let t = Instant::now();
    let mut chk = 0u64;
    for &idx in &indices {
      // SAFETY: indices are pre-generated within bounds
      chk ^= unsafe { pc.get_unchecked(idx) };
    }
    black_box(chk);
    black_box(chk);
    let get_mops = nq as f64 / 1e6 / t.elapsed().as_secs_f64();

    // Latency P99
    let mut latencies = Vec::with_capacity(nq);
    let mut rng = StdRng::seed_from_u64(SEED + 1);
    for _ in 0..nq {
      let idx = rng.random_range(0..n);
      let start = Instant::now();
      let val = pc.get(idx).unwrap_or(0);
      let d = start.elapsed();
      black_box(val);
      latencies.push(d.as_nanos() as f64);
    }
    latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let latency_p99_ns = latencies[(latencies.len() as f64 * 0.99) as usize];

    // Sequential iteration
    // Sequential iteration
    let t = Instant::now();
    let mut chk = 0u64;
    for val in pc.iter() {
      chk ^= val;
    }
    black_box(chk);
    let iter_mops = n as f64 / 1e6 / t.elapsed().as_secs_f64();

    // Reverse iteration
    let t = Instant::now();
    let mut chk = 0u64;
    for val in pc.rev_iter() {
      chk ^= val;
    }
    black_box(chk);
    let rev_mops = Some(n as f64 / 1e6 / t.elapsed().as_secs_f64());

    Metrics {
      size_mb: total as f64 / 1024.0 / 1024.0,
      ratio_pct: total as f64 / orig as f64 * 100.0,
      build_mops,
      get_mops,
      iter_mops,
      rev_mops,
      latency_p99_ns,
    }
  }
}
