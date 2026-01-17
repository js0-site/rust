// schnellru adapter / schnellru 适配器
// Item count based capacity with ByLength limiter
// 基于条目数的容量，使用 ByLength 限制器

use crate::common::LruBench;

const LIB: &str = "schnellru";

/// schnellru::LruMap adapter with ByLength limiter
/// schnellru::LruMap 适配器，使用 ByLength 限制器
pub struct SchnellruAdapter {
  cache: schnellru::LruMap<Vec<u8>, Vec<u8>, schnellru::ByLength>,
}

impl LruBench for SchnellruAdapter {
  const NAME: &'static str = LIB;

  fn new(mem_budget: usize, _target_mem_mb: u64) -> Self {
    // mem_budget is already the item count capacity
    // mem_budget 已经是条目数容量
    Self {
      cache: schnellru::LruMap::new(schnellru::ByLength::new((mem_budget as u32).max(1))),
    }
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
