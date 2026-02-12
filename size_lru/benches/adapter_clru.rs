// clru adapter / clru 适配器
// Weight-aware with WeightScale = key.len() + val.len()
// 权重感知，WeightScale = key.len() + val.len()

use std::{collections::hash_map::RandomState, num::NonZeroUsize};

use crate::common::LruBench;

const LIB: &str = "clru";

/// Weight scale: key.len() + val.len()
/// 权重计算：key.len() + val.len()
struct KvWeight;

impl clru::WeightScale<Vec<u8>, Vec<u8>> for KvWeight {
  fn weight(&self, key: &Vec<u8>, val: &Vec<u8>) -> usize {
    key.len() + val.len()
  }
}

/// clru::CLruCache adapter with weight-based eviction
/// clru::CLruCache 适配器，基于权重淘汰
pub struct ClruAdapter {
  cache: clru::CLruCache<Vec<u8>, Vec<u8>, RandomState, KvWeight>,
}

impl LruBench for ClruAdapter {
  const NAME: &'static str = LIB;

  fn new(mem_budget: usize, _target_mem_mb: u64) -> Self {
    // mem_budget is already the item count capacity
    // mem_budget 已经是条目数容量
    Self {
      cache: clru::CLruCache::with_config(
        clru::CLruCacheConfig::new(NonZeroUsize::new(mem_budget).expect("cap > 0"))
          .with_scale(KvWeight),
      ),
    }
  }

  fn set(&mut self, key: &[u8], val: &[u8]) {
    let _ = self.cache.put_with_weight(key.to_vec(), val.to_vec());
  }

  fn get(&mut self, key: &[u8]) -> bool {
    self.cache.get(&key.to_vec()).is_some()
  }

  fn del(&mut self, key: &[u8]) -> bool {
    self.cache.pop(&key.to_vec()).is_some()
  }
}
