// hashlink adapter / hashlink 适配器
// Item count based capacity (no weight support)
// 基于条目数的容量（不支持权重）

use crate::common::{LruBench, calibrated_cap};

const LIB: &str = "hashlink";
const AVG_ITEM_SIZE: usize = 200;

/// hashlink::LruCache adapter with item count capacity
/// hashlink::LruCache 适配器，使用条目数容量
pub struct HashlinkAdapter {
  cache: hashlink::LruCache<Vec<u8>, Vec<u8>>,
}

impl LruBench for HashlinkAdapter {
  fn new(mem_budget: usize, target_mem_mb: u64) -> Self {
    let cap = calibrated_cap(LIB, mem_budget, target_mem_mb) / AVG_ITEM_SIZE;
    Self {
      cache: hashlink::LruCache::new(cap.max(1)),
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
