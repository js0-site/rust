// lru adapter / lru 适配器
// Item count based capacity (no weight support)
// 基于条目数的容量（不支持权重）

use std::num::NonZeroUsize;

use crate::common::LruBench;

const LIB: &str = "lru";

/// lru::LruCache adapter with item count capacity
/// lru::LruCache 适配器，使用条目数容量
pub struct LruAdapter {
  cache: lru::LruCache<Vec<u8>, Vec<u8>>,
}

impl LruBench for LruAdapter {
  const NAME: &'static str = LIB;

  fn new(mem_budget: usize, _target_mem_mb: u64) -> Self {
    // mem_budget is already the item count capacity
    // mem_budget 已经是条目数容量
    Self {
      cache: lru::LruCache::new(NonZeroUsize::new(mem_budget.max(1)).expect("cap > 0")),
    }
  }

  fn set(&mut self, key: &[u8], val: &[u8]) {
    self.cache.put(key.to_vec(), val.to_vec());
  }

  fn get(&mut self, key: &[u8]) -> bool {
    self.cache.get(&key.to_vec()).is_some()
  }

  fn del(&mut self, key: &[u8]) -> bool {
    self.cache.pop(&key.to_vec()).is_some()
  }
}
