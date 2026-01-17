use std::time::Instant;

use rand::prelude::*;

use crate::library::{Library, Metrics, N_QUERIES, SEED};

pub struct VecLib;

impl Library for VecLib {
  const NAME: &'static str = "[u64]";

  fn measure(data: &[u64]) -> Metrics {
    let n = data.len();
    let orig = n * 8;

    // Build (Clone to Box)
    let t = Instant::now();
    let boxed: Box<[u64]> = data.into();
    let build_mops = n as f64 / 1e6 / t.elapsed().as_secs_f64();

    let total = boxed.len() * 8; // Pure data size

    // Random access (direct get_unchecked)
    // Pre-generate indices to exclude RNG cost
    let mut rng = StdRng::seed_from_u64(SEED);
    let nq = N_QUERIES.min(n);
    let indices: Vec<usize> = (0..nq).map(|_| rng.random_range(0..n)).collect();

    let t = Instant::now();
    let mut chk = 0usize;
    for &idx in &indices {
      // SAFETY: idx is bounded by 0..n
      let val = unsafe { *boxed.get_unchecked(idx) };
      chk ^= val as usize;
    }
    std::hint::black_box(chk);
    let get_mops = nq as f64 / 1e6 / t.elapsed().as_secs_f64();

    // Latency P99
    let mut latencies = Vec::with_capacity(nq);
    let mut rng = StdRng::seed_from_u64(SEED + 1);
    for _ in 0..nq {
      let idx = rng.random_range(0..n);
      let start = Instant::now();
      let val = unsafe { *boxed.get_unchecked(idx) };
      let d = start.elapsed();
      std::hint::black_box(val);
      latencies.push(d.as_nanos() as f64);
    }
    latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let latency_p99_ns = latencies[(latencies.len() as f64 * 0.99) as usize];

    // Sequential iteration (unsafe)
    let t = Instant::now();
    let mut chk = 0u64;
    let ptr = boxed.as_ptr();
    for i in 0..n {
      // SAFETY: i is within 0..n
      chk ^= unsafe { *ptr.add(i) };
    }
    std::hint::black_box(chk);
    let iter_mops = n as f64 / 1e6 / t.elapsed().as_secs_f64();

    // Reverse iteration (unsafe)
    let t = Instant::now();
    let mut chk = 0u64;
    let ptr = boxed.as_ptr();
    for i in 0..n {
      // SAFETY: i is within 0..n
      chk ^= unsafe { *ptr.add(n - 1 - i) };
    }
    std::hint::black_box(chk);
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
