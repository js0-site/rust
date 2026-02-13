// hashlink adapter / hashlink 适配器
// Item count based capacity (no weight support)
// 基于条目数的容量（不支持权重）

use crate::common::LruBench;

const LIB: &str = "hashlink";

/// hashlink::LruCache adapter with item count capacity
/// hashlink::LruCache 适配器，使用条目数容量
pub struct HashlinkAdapter {
  cache: hashlink::LruCache<Vec<u8>, Vec<u8>>,
}

impl LruBench for HashlinkAdapter {
  const NAME: &'static str = LIB;

  fn new(mem_budget: usize, _target_mem_mb: u64) -> Self {
    // mem_budget is already the item count capacity
    // mem_budget 已经是条目数容量
    Self {
      cache: hashlink::LruCache::new(mem_budget.max(1)),
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
