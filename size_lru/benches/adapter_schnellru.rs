// schnellru adapter / schnellru 适配器
// Item count based capacity with ByLength limiter
// 基于条目数的容量，使用 ByLength 限制器

use crate::common::{LruBench, calibrated_cap};

const LIB: &str = "schnellru";
const AVG_ITEM_SIZE: usize = 200;

/// schnellru::LruMap adapter with ByLength limiter
/// schnellru::LruMap 适配器，使用 ByLength 限制器
pub struct SchnellruAdapter {
  cache: schnellru::LruMap<Vec<u8>, Vec<u8>, schnellru::ByLength>,
}

impl LruBench for SchnellruAdapter {
  fn new(mem_budget: usize, target_mem_mb: u64) -> Self {
    let cap = (calibrated_cap(LIB, mem_budget, target_mem_mb) / AVG_ITEM_SIZE) as u32;
    Self {
      cache: schnellru::LruMap::new(schnellru::ByLength::new(cap.max(1))),
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
    self.cache.remove(&key.to_vec()).is_some()
  }
}
