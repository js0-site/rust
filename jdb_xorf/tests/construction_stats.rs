use std::time::Instant;

use jdb_xorf::{Bf8, Filter};
use rand::{RngExt, SeedableRng, rngs::StdRng};

#[test]
fn test_construction_success_stats() {
  let num_trials = 1000;
  // 使用足够大的键集来挑战算法
  let num_keys = 100_000;
  let mut rng = StdRng::seed_from_u64(42);

  println!("\n=== 构建成功率统计 ===");
  println!("测试轮数: {num_trials}, 每轮键数量: {num_keys}");
  println!("正在运行测试...");

  // 预先生成所有键，避免包含生成时间
  let mut all_test_keys = Vec::with_capacity(num_trials);
  for _ in 0..num_trials {
    let keys: Vec<u64> = (0..num_keys).map(|_| rng.random()).collect();
    all_test_keys.push(keys);
  }

  let start = Instant::now();

  let mut success_count = 0;
  for keys in &all_test_keys {
    // 构造过滤器
    let filter = Bf8::from(keys);

    // 检查过滤器是否有效（非空）
    if filter.len() > 0 {
      success_count += 1;
    }
  }

  let duration = start.elapsed();
  let avg_duration = duration / num_trials as u32;

  println!("\n测试结果:");
  println!("总耗时: {duration:?}");
  println!("平均每次构建耗时: {avg_duration:?}");

  let success_rate = (success_count as f64 / num_trials as f64) * 100.0;
  println!("构建成功率: {success_rate:.2}%");

  assert!(
    success_rate > 99.0,
    "构建成功率应非常高 (>99%), 实际: {success_rate:.2}%"
  );
}
