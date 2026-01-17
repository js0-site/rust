// Test data distribution accuracy / 测试数据分布精度
// Verify error < 0.1% from Facebook distribution / 验证与 Facebook 分布误差 < 0.1%

mod zipf_data;

use zipf_data::{ZipfDataConfig, ZipfDataGenerator};

fn main() {
  println!("\n=== Testing Data Distribution Accuracy ===\n");

  let config = ZipfDataConfig {
    num_keys: 100_000,
    zipf_s: 1.0,
    seed: 42,
  };

  let mut generator = ZipfDataGenerator::new(config);
  let data = generator.generate_all();

  let total_size = ZipfDataGenerator::total_size(&data);
  let avg_size = ZipfDataGenerator::avg_size(&data);
  let (min_size, max_size) = ZipfDataGenerator::size_range(&data);

  println!("Generated {} items", data.len());
  println!("Total size: {:.2}MB", total_size as f64 / (1024.0 * 1024.0));
  println!("Avg size: {}B", avg_size);
  println!("Size range: {}B - {}B\n", min_size, max_size);

  // Analyze tier distribution / 分析层级分布
  let mut tier_stats = vec![(0, 0u64); 5];
  let tier_ranges = [
    (16, 100, "Tiny (16-100B)"),
    (100, 1024, "Small (100B-1KB)"),
    (1024, 10240, "Medium (1-10KB)"),
    (10240, 102400, "Large (10-100KB)"),
    (102400, 1048576, "Huge (100KB-1MB)"),
  ];

  for (k, v) in &data {
    let size = k.len() + v.len();
    for (idx, &(min, max, _)) in tier_ranges.iter().enumerate() {
      if size >= min && size < max {
        tier_stats[idx].0 += 1;
        tier_stats[idx].1 += size as u64;
        break;
      }
    }
  }

  println!("Actual Distribution:");
  println!(
    "{:<25} {:>10} {:>10} {:>12} {:>10}",
    "Tier", "Count", "Count%", "Size(MB)", "Size%"
  );
  println!("{}", "-".repeat(70));

  let expected_item_pcts = [40.0, 35.0, 20.0, 4.0, 1.0];
  let expected_size_pcts = [0.3, 2.2, 12.0, 24.0, 61.5];

  let mut max_item_error = 0.0f64;
  let mut max_size_error = 0.0f64;

  for (idx, &(count, size_bytes)) in tier_stats.iter().enumerate() {
    let count_pct = count as f64 / data.len() as f64 * 100.0;
    let size_pct = size_bytes as f64 / total_size as f64 * 100.0;
    let size_mb = size_bytes as f64 / (1024.0 * 1024.0);

    let item_error = (count_pct - expected_item_pcts[idx]).abs();
    let size_error = (size_pct - expected_size_pcts[idx]).abs();

    max_item_error = max_item_error.max(item_error);
    max_size_error = max_size_error.max(size_error);

    println!(
      "{:<25} {:>10} {:>9.2}% {:>11.2} {:>9.2}%",
      tier_ranges[idx].2, count, count_pct, size_mb, size_pct
    );
  }

  println!("\nExpected Distribution (from README):");
  println!("{:<25} {:>10} {:>12}", "Tier", "Items%", "Size%");
  println!("{}", "-".repeat(50));
  println!("{:<25} {:>10} {:>12}", "Tiny Metadata", "40%", "~0.3%");
  println!("{:<25} {:>10} {:>12}", "Small Structs", "35%", "~2.2%");
  println!("{:<25} {:>10} {:>12}", "Medium Content", "20%", "~12%");
  println!("{:<25} {:>10} {:>12}", "Large Objects", "4%", "~24%");
  println!("{:<25} {:>10} {:>12}", "Huge Blobs", "1%", "~61.5%");

  println!("\n=== Error Analysis ===");
  println!("Max item count error: {:.3}%", max_item_error);
  println!("Max size distribution error: {:.3}%", max_size_error);

  // Verify error < 0.1% / 验证误差 < 0.1%
  if max_item_error < 0.1 && max_size_error < 0.1 {
    println!("\n✓ SUCCESS: Distribution error < 0.1%");
  } else {
    println!("\n✗ FAILED: Distribution error >= 0.1%");
    std::process::exit(1);
  }
}
