// moka adapter / moka 适配器
// Weight-aware with weigher = key.len() + val.len()
// 权重感知，weigher = key.len() + val.len()

use crate::common::{LruBench, calibrated_cap};

const LIB: &str = "moka";

/// moka::sync::Cache adapter with weight-based eviction
/// moka::sync::Cache 适配器，基于权重淘汰
pub struct MokaAdapter {
  cache: moka::sync::Cache<Vec<u8>, Vec<u8>>,
}

impl LruBench for MokaAdapter {
  fn new(mem_budget: usize, target_mem_mb: u64) -> Self {
    let cap = calibrated_cap(LIB, mem_budget, target_mem_mb);
    Self {
      cache: moka::sync::Cache::builder()
        .weigher(|k: &Vec<u8>, v: &Vec<u8>| (k.len() + v.len()) as u32)
        .max_capacity(cap as u64)
        .build(),
    }
  }

  fn name(&self) -> &'static str {
    LIB
  }

  fn set(&mut self, key: &[u8], val: &[u8]) {
    self.cache.insert(key.to_vec(), val.to_vec());
  }

  fn get(&mut self, key: &[u8]) -> bool {
    self.cache.get(&key.to_vec()).is_some()
  }

  fn del(&mut self, key: &[u8]) -> bool {
    self.cache.invalidate(&key.to_vec());
    true
  }
}
