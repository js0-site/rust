use std::time::Instant;

use rand::prelude::*;
use sucds::{Serializable, mii_sequences::EliasFanoBuilder};

use crate::library::{Library, Metrics, N_QUERIES, SEED};

pub struct SucdsLib;
impl Library for SucdsLib {
  const NAME: &'static str = "sucds";

  fn measure(data: &[u64]) -> Metrics {
    let n = data.len();
    let orig = n * 8;

    let t = Instant::now();
    let max_val = *data.last().unwrap() as usize + 1;
    let mut efb = EliasFanoBuilder::new(max_val, n).unwrap();
    efb.extend(data.iter().map(|&x| x as usize)).unwrap();
    let ef = efb.build();
    let build_mops = n as f64 / 1e6 / t.elapsed().as_secs_f64();

    let size = ef.size_in_bytes();

    let mut rng = StdRng::seed_from_u64(SEED);
    let nq = N_QUERIES.min(n);
    let indices: Vec<usize> = (0..nq).map(|_| rng.random_range(0..n)).collect();

    let t = Instant::now();
    let mut chk = 0usize;
    for &idx in &indices {
      chk ^= ef.select(idx).unwrap_or(0);
    }
    std::hint::black_box(chk);
    let get_mops = nq as f64 / 1e6 / t.elapsed().as_secs_f64();

    // Latency P99
    let mut latencies = Vec::with_capacity(nq);
    let mut rng = StdRng::seed_from_u64(SEED + 1);
    for _ in 0..nq {
      let idx = rng.random_range(0..n);
      let start = Instant::now();
      let val = ef.select(idx).unwrap_or(0);
      let d = start.elapsed();
      std::hint::black_box(val);
      latencies.push(d.as_nanos() as f64);
    }
    latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let latency_p99_ns = latencies[(latencies.len() as f64 * 0.99) as usize];

    let t = Instant::now();
    let mut chk = 0usize;
    for val in ef.iter(0) {
      chk ^= val;
    }
    std::hint::black_box(chk);
    let iter_mops = n as f64 / 1e6 / t.elapsed().as_secs_f64();

    Metrics {
      size_mb: size as f64 / 1024.0 / 1024.0,
      ratio_pct: size as f64 / orig as f64 * 100.0,
      build_mops,
      get_mops,
      iter_mops,
      rev_mops: None,
      latency_p99_ns,
    }
  }
}
