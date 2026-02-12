use crate::common::LruBench;

const LIB: &str = "size_lru";

/// size_lru::Lhd adapter with weight = key.len() + val.len()
/// size_lru::Lhd 适配器，权重 = key.len() + val.len()
pub struct SizeLruAdapter {
  cache: size_lru::Lhd<Vec<u8>, Vec<u8>>,
}

impl LruBench for SizeLruAdapter {
  const NAME: &'static str = LIB;

  fn new(mem_budget: usize, _target_mem_mb: u64) -> Self {
    // Use mem_budget directly as capacity (in bytes)
    // 直接使用 mem_budget 作为容量（字节）
    Self {
      cache: size_lru::Lhd::new(mem_budget),
    }
  }

  fn set(&mut self, key: &[u8], val: &[u8]) {
    let weight = (key.len() + val.len()) as u32;
    self.cache.set(key.to_vec(), val.to_vec(), weight);
  }

  fn get(&mut self, key: &[u8]) -> bool {
    self.cache.get(key).is_some()
  }

  fn del(&mut self, key: &[u8]) -> bool {
    // Check existence before removal or eviction
    // 删除/淘汰前检查是否存在
    let existed = self.cache.get(key).is_some();
    self.cache.rm(key);
    existed
  }
}
