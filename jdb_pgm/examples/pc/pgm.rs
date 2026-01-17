use std::time::Instant;

use jdb_pgm::Pc;
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
    let pc = Pc::new(data, jdb_pgm::pc::types::DEFAULT_EPSILON);
    let encoded = pc.dump();
    let build_mops = n as f64 / 1e6 / t.elapsed().as_secs_f64();

    let total = encoded.len();

    // Validate load
    let loaded = Pc::load(&encoded);
    assert_eq!(loaded.len, pc.len);
    // Note: get(0) might differ slightly due to float precision if data[0] is special,
    // but should be fine for u64.
    assert_eq!(loaded.get(0).unwrap(), pc.get(0).unwrap());

    // Random access
    let mut rng = StdRng::seed_from_u64(SEED);
    let nq = N_QUERIES.min(n);
    let t = Instant::now();
    let mut chk = 0u64;
    for _ in 0..nq {
      chk ^= pc.get(rng.random_range(0..n)).unwrap_or(0);
    }
    std::hint::black_box(chk);
    let get_mops = nq as f64 / 1e6 / t.elapsed().as_secs_f64();

    // Sequential iteration
    let t = Instant::now();
    let mut chk = 0u64;
    for val in pc.iter() {
      chk ^= val;
    }
    std::hint::black_box(chk);
    let iter_mops = n as f64 / 1e6 / t.elapsed().as_secs_f64();

    // Reverse iteration
    let t = Instant::now();
    let mut chk = 0u64;
    for val in pc.rev_iter() {
      chk ^= val;
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
    }
  }
}
