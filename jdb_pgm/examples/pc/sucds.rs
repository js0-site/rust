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
    let t = Instant::now();
    let mut chk = 0usize;
    for _ in 0..nq {
      chk ^= ef.select(rng.random_range(0..n)).unwrap_or(0);
    }
    std::hint::black_box(chk);
    let get_mops = nq as f64 / 1e6 / t.elapsed().as_secs_f64();

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
    }
  }
}
