//! Binary search benchmark
//! 二分查找评测

use jdb_pgm::bench_common::Benchmarkable;

pub struct BinarySearch;

impl Benchmarkable for BinarySearch {
  const NAME: &'static str = "binary_search";

  fn build(_data: &[u64], _epsilon: Option<usize>) -> Self {
    Self
  }

  fn query(&self, data: &[u64], key: u64) -> Option<usize> {
    data.binary_search(&key).ok()
  }
}
