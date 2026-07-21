use std::time::Instant;

use jdb_xorf::Bf8;
use rand::{prelude::*, rngs::StdRng};

#[test]
fn test_construction_success_stats() {
  let num_trials = 1000;
  // 使用足够大的键集来挑战算法
  let num_keys = 100_000;
  let mut rng = StdRng::seed_from_u64(42);

  // 统计一次性成功的次数
  let mut first_try_success_count = 0;
  let mut max_seed = 0;
  let mut total_attempts = 0;

  println!("\n=== 构建成功率统计 ===");
  println!("测试轮数: {}, 每轮键数量: {}", num_trials, num_keys);
  println!("正在运行测试...");

  // 预先生成所有键，避免包含生成时间
  let mut all_test_keys = Vec::with_capacity(num_trials);
  for _ in 0..num_trials {
    let keys: Vec<u64> = (0..num_keys).map(|_| rng.random()).collect();
    all_test_keys.push(keys);
  }

  let start = Instant::now();

  for keys in &all_test_keys {
    // 构造过滤器
    let filter = Bf8::from(keys);

    // 检查 seed 字段
    // 注意：make 内部逻辑是 seed 从 1 开始，每次 +1
    // 所以 filter.desc.seed 就是尝试的次数
    let attempts = filter.desc.seed;

    if attempts == 1 {
      first_try_success_count += 1;
    }

    if attempts > max_seed {
      max_seed = attempts;
    }
    total_attempts += attempts;
  }

  let duration = start.elapsed();

  println!("\n测试结果:");
  println!("总耗时: {:?}", duration);
  println!("平均每次构建耗时: {:?}", duration / num_trials as u32);
  println!("最大尝试次数: {}", max_seed);
  println!(
    "平均尝试次数: {:.4}",
    total_attempts as f64 / num_trials as f64
  );

  let first_try_rate = (first_try_success_count as f64 / num_trials as f64) * 100.0;
  println!("\n一次性构建成功率: {:.2}%", first_try_rate);

  assert!(first_try_rate > 95.0, "单次构建成功率应非常高 (>95%)");
  // 对于 Binary Fuse，期望是非常接近 100% 的
}
