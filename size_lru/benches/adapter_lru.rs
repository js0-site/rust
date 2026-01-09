// lru adapter / lru 适配器
// Item count based capacity (no weight support)
// 基于条目数的容量（不支持权重）

use std::num::NonZeroUsize;

use crate::common::{LruBench, calibrated_cap};

const LIB: &str = "lru";

/// lru::LruCache adapter with item count capacity
/// lru::LruCache 适配器，使用条目数容量
pub struct LruAdapter {
  cache: lru::LruCache<Vec<u8>, Vec<u8>>,
}

impl LruBench for LruAdapter {
  fn new(mem_budget: usize, target_mem_mb: u64) -> Self {
    // Use calibrated_cap for item count estimation
    // 使用 calibrated_cap 估算条目数
    let cap = calibrated_cap(LIB, mem_budget, target_mem_mb);
    Self {
      cache: lru::LruCache::new(NonZeroUsize::new(cap.max(1)).expect("cap > 0")),
    }
  }

  fn name(&self) -> &'static str {
    LIB
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
