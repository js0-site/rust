//! FPR 压力测试：使用相似字符串验证哈希函数质量
//! FPR stress test: validate hash function quality with similar strings

use std::hash::Hasher;

use rapidhash::RapidHashSet as HashSet;

use jdb_xorf::{Bf8, Bf16, Bf32, Filter, RapidHasher};

/// 使用 RapidHasher 将字符串转换为 u64
/// Convert string to u64 using RapidHasher
fn hash_string(s: &str) -> u64 {
  let mut hasher = RapidHasher::default();
  hasher.write(s.as_bytes());
  hasher.finish()
}

/// 生成相似的顺序字符串键
/// Generate similar sequential string keys
fn gen_similar_keys(prefix: &str, count: usize) -> Vec<u64> {
  (0..count)
    .map(|i| hash_string(&format!("{prefix}{i}")))
    .collect()
}

/// 测试指定前缀的 FPR
/// Test FPR with specified prefix
fn test_fpr<F: Filter<u64>>(
  filter: &F,
  key_set: &HashSet<u64>,
  test_prefix: &str,
  test_count: usize,
) -> f64 {
  let mut fp_count = 0;
  let mut tested = 0;

  for i in 0..test_count {
    let key = hash_string(&format!("{test_prefix}{i}"));
    if !key_set.contains(&key) {
      tested += 1;
      if filter.has(&key) {
        fp_count += 1;
      }
    }
  }

  if tested > 0 {
    fp_count as f64 / tested as f64 * 100.0
  } else {
    0.0
  }
}

#[test]
fn test_similar_strings_fpr_bfuse8() {
  const KEY_COUNT: usize = 100_000;
  const TEST_COUNT: usize = 1_000_000;

  // 构建键集合
  let keys = gen_similar_keys("user_id_", KEY_COUNT);
  let key_set: HashSet<u64> = keys.iter().copied().collect();
  let filter = Bf8::from(&keys);

  // 测试不同模式的相似字符串
  let patterns = [
    ("user_id_", "顺序相似 (相同前缀)"),
    ("user_ID_", "大小写变体"),
    ("userid_", "无下划线变体"),
    ("user_id", "无尾下划线"),
    ("test_key_", "完全不同前缀"),
    ("", "纯数字"),
  ];

  println!("\n=== Bf8 相似字符串 FPR 测试 ===");
  println!("键数量: {KEY_COUNT}, 测试数量: {TEST_COUNT}");
  println!("理论 FPR (u8): ~0.39%\n");

  for (prefix, desc) in patterns {
    let fpr = test_fpr(&filter, &key_set, prefix, TEST_COUNT);
    println!("{desc}: {fpr:.4}%");

    // FPR 应该在合理范围内 (0.2% - 0.6%)
    assert!(fpr < 1.0, "FPR 过高 ({fpr:.4}%) for pattern '{prefix}'");
  }
}

#[test]
fn test_similar_strings_fpr_bfuse16() {
  const KEY_COUNT: usize = 100_000;
  const TEST_COUNT: usize = 1_000_000;

  let keys = gen_similar_keys("item_", KEY_COUNT);
  let key_set: HashSet<u64> = keys.iter().copied().collect();
  let filter = Bf16::from(&keys);

  println!("\n=== Bf16 相似字符串 FPR 测试 ===");
  println!("键数量: {KEY_COUNT}, 测试数量: {TEST_COUNT}");
  println!("理论 FPR (u16): ~0.0015%\n");

  let patterns = [
    ("item_", "顺序相似"),
    ("ITEM_", "大写变体"),
    ("product_", "不同前缀"),
  ];

  for (prefix, desc) in patterns {
    let fpr = test_fpr(&filter, &key_set, prefix, TEST_COUNT);
    println!("{desc}: {fpr:.6}%");

    // u16 FPR 应该非常低 (< 0.01%)
    assert!(fpr < 0.1, "FPR 过高 ({fpr:.6}%) for pattern '{prefix}'");
  }
}

#[test]
fn test_similar_strings_fpr_bfuse32() {
  const KEY_COUNT: usize = 100_000;
  const TEST_COUNT: usize = 1_000_000;

  let keys = gen_similar_keys("record_", KEY_COUNT);
  let key_set: HashSet<u64> = keys.iter().copied().collect();
  let filter = Bf32::from(&keys);

  println!("\n=== Bf32 相似字符串 FPR 测试 ===");
  println!("键数量: {KEY_COUNT}, 测试数量: {TEST_COUNT}");
  println!("理论 FPR (u32): ~0.0000002%\n");

  let patterns = [
    ("record_", "顺序相似"),
    ("RECORD_", "大写变体"),
    ("entry_", "不同前缀"),
  ];

  for (prefix, desc) in patterns {
    let fpr = test_fpr(&filter, &key_set, prefix, TEST_COUNT);
    println!("{desc}: {fpr:.8}%");

    // u32 FPR 应该接近 0
    assert!(fpr < 0.001, "FPR 过高 ({fpr:.8}%) for pattern '{prefix}'");
  }
}

#[test]
fn test_sequential_integers() {
  // 测试最坏情况：完全顺序的整数键
  // Test worst case: purely sequential integer keys
  const KEY_COUNT: usize = 100_000;
  const TEST_COUNT: usize = 1_000_000;

  // 使用顺序整数作为键
  let keys: Vec<u64> = (0..KEY_COUNT as u64).collect();
  let key_set: HashSet<u64> = keys.iter().copied().collect();
  let filter = Bf8::from(&keys);

  // 测试不在集合中的顺序整数
  let mut fp_count = 0;
  let mut tested = 0;
  for i in KEY_COUNT as u64..(KEY_COUNT as u64 + TEST_COUNT as u64) {
    if !key_set.contains(&i) {
      tested += 1;
      if filter.has(&i) {
        fp_count += 1;
      }
    }
  }

  let fpr = fp_count as f64 / tested as f64 * 100.0;
  println!("\n=== 顺序整数 FPR 测试 (最坏情况) ===");
  println!(
    "键: 0..{KEY_COUNT}, 测试: {KEY_COUNT}..{}",
    KEY_COUNT + TEST_COUNT
  );
  println!("FPR: {fpr:.4}%");
  println!("理论 FPR: ~0.39%");

  // 即使是顺序整数，FPR 也应该在合理范围内
  assert!(
    fpr < 1.0,
    "顺序整数 FPR 过高: {fpr:.4}%，可能哈希函数质量不足"
  );
}
