// 测试 PgmIndex 的 bitcode 序列化

#[cfg(feature = "bitcode")]
use jdb_pgm::PgmIndex;

#[cfg(feature = "bitcode")]
fn main() {
  // 创建测试数据
  let data: Vec<u64> = (0..10_000).collect();
  let original_index = PgmIndex::load(data.clone(), 64, true).expect("build pgm");

  println!("原始索引统计:");
  println!("  段数: {}", original_index.segment_count());
  println!("  平均段大小: {:.2}", original_index.avg_segment_size());
  println!("  内存使用: {} bytes", original_index.memory_usage());

  // 测试查找
  let test_keys = vec![0u64, 1234, 5678, 9999];
  println!("\n原始索引查找结果:");
  for &key in &test_keys {
    println!("  key {}: {:?}", key, original_index.get(key));
  }

  // 编码
  let encoded = bitcode::encode(&original_index);
  println!("\n编码成功，大小: {} bytes", encoded.len());

  // 解码
  let decoded_index: PgmIndex<u64> = bitcode::decode(&encoded).unwrap();
  println!("解码成功");

  // 验证解码后的索引
  println!("\n解码后索引统计:");
  println!("  段数: {}", decoded_index.segment_count());
  println!("  平均段大小: {:.2}", decoded_index.avg_segment_size());
  println!("  内存使用: {} bytes", decoded_index.memory_usage());

  // 验证查找结果一致
  println!("\n解码后索引查找结果:");
  for &key in &test_keys {
    let original_result = original_index.get(key);
    let decoded_result = decoded_index.get(key);
    println!(
      "  key {}: 原始={:?}, 解码={:?}",
      key, original_result, decoded_result
    );
    assert_eq!(
      original_result, decoded_result,
      "key {} 的查找结果不一致",
      key
    );
  }

  println!("\n✅ 所有测试通过！bitcode 序列化功能正常工作");
}

#[cfg(not(feature = "bitcode"))]
fn main() {
  println!("请使用 --features bitcode 运行此示例");
  println!("例如: cargo run --example test_pgm_bitcode --features bitcode");
}
