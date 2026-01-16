//! Cache trait definition
//! 缓存 trait 定义

/// Cache trait for basic operations
/// 缓存基本操作 trait
///
/// # Complexity
/// 复杂度
///
/// All implementations should provide:
/// 所有实现应提供：
/// - get: O(1)
/// - set: O(1) amortized
/// - rm: O(1)
pub trait Cache<K, V> {
  /// Get value by key
  /// 按键获取值
  fn get(&mut self, key: &K) -> Option<&V>;

  /// Insert key-value pair
  /// 插入键值对
  fn set(&mut self, key: K, val: V);

  /// Remove by key
  /// 按键删除
  fn rm(&mut self, key: &K);
}
