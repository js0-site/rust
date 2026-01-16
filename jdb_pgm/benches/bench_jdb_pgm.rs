//! jdb_pgm benchmark
//! jdb_pgm 评测

use jdb_pgm::{Pgm, bench_common::Benchmarkable};

/// Linear scan threshold
/// 线性扫描阈值
const LINEAR_THRESHOLD: usize = 32;

pub struct JdbPgm {
  pgm: Pgm<u64>,
}

impl Benchmarkable for JdbPgm {
  const NAME: &'static str = "jdb_pgm";

  fn build(data: &[u64], epsilon: Option<usize>) -> Self {
    let pgm = Pgm::new(data, epsilon.unwrap_or(64), false).unwrap();
    Self { pgm }
  }

  fn query(&self, data: &[u64], key: u64) -> Option<usize> {
    let (lo, hi) = self.pgm.predict_range(key);
    let hi = hi.min(data.len());
    let len = hi - lo;

    if len <= LINEAR_THRESHOLD {
      // Linear scan for small ranges
      // 小范围线性扫描
      for i in lo..hi {
        let v = unsafe { *data.get_unchecked(i) };
        if v == key {
          return Some(i);
        }
        if v > key {
          return None;
        }
      }
      None
    } else {
      // Binary search for large ranges
      // 大范围二分查找
      unsafe { data.get_unchecked(lo..hi) }
        .binary_search(&key)
        .ok()
        .map(|p| lo + p)
    }
  }
}
