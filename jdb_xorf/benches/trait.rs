// Benchmark trait definition
// 评测 trait 定义

/// Unified benchmark interface for all filter implementations
/// 所有过滤器实现的统一评测接口
pub trait FilterBench {
  /// Filter implementation name
  /// 过滤器实现名称
  const NAME: &'static str;

  /// Build filter from keys
  /// 从键构建过滤器
  fn build(keys: &[u64]) -> Self;

  /// Check if filter contains key
  /// 检查过滤器是否包含键
  fn contains(&self, key: &u64) -> bool;

  /// Get memory usage in bytes
  /// 获取内存占用（字节）
  fn memory_usage(&self) -> usize;
}
