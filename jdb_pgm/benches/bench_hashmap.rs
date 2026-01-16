//! HashMap benchmark
//! HashMap 评测

use std::collections::HashMap;

use jdb_pgm::bench_common::Benchmarkable;

pub struct HashMapIndex {
  map: HashMap<u64, usize>,
}

impl Benchmarkable for HashMapIndex {
  const NAME: &'static str = "hashmap";

  fn build(data: &[u64], _epsilon: Option<usize>) -> Self {
    let map = data.iter().enumerate().map(|(i, &v)| (v, i)).collect();
    Self { map }
  }

  fn query(&self, _data: &[u64], key: u64) -> Option<usize> {
    self.map.get(&key).copied()
  }
}
