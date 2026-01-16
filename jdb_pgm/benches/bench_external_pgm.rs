//! External pgm_index benchmark
//! 外部 pgm_index 评测

use jdb_pgm::bench_common::Benchmarkable;
use pgm_index as external_pgm;

pub struct ExternalPgm {
  index: external_pgm::PGMIndex<u64>,
}

impl Benchmarkable for ExternalPgm {
  const NAME: &'static str = "external_pgm";

  fn build(data: &[u64], epsilon: Option<usize>) -> Self {
    let index = external_pgm::PGMIndex::new(data.to_vec(), epsilon.unwrap_or(64));
    Self { index }
  }

  fn query(&self, _data: &[u64], key: u64) -> Option<usize> {
    self.index.get(key)
  }
}
