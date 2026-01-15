// Zipf-distributed data generator / Zipf分布数据生成器
// Generates key-value pairs with varying sizes following Zipf distribution
// 生成大小变化遵循Zipf分布的键值对
// Based on Facebook USR/APP/VAR pools and Twitter/Meta traces
// 基于 Facebook USR/APP/VAR 池和 Twitter/Meta 追踪数据

/// Size tier for realistic distribution / 真实分布的大小层级
#[derive(Clone, Copy)]
struct SizeTier {
  /// Size range in bytes / 字节大小范围
  range: (usize, usize),
  /// Percentage of items / 条目百分比
  item_pct: f64,
  /// Percentage of total size / 总大小百分比
  size_pct: f64,
}

// Five-tier distribution based on Facebook/Twitter data
// 基于 Facebook/Twitter 数据的五层分布
// Adjusted to match exact percentages from documentation (error < 0.1%)
// 调整为精确匹配文档中的百分比（误差 < 0.1%）
const SIZE_TIERS: [SizeTier; 5] = [
  SizeTier {
    range: (16, 100),      // Tiny Metadata
    item_pct: 0.40,        // 40% items
    size_pct: 0.003,       // ~0.3% size
  },
  SizeTier {
    range: (100, 1024),    // Small Structs
    item_pct: 0.35,        // 35% items
    size_pct: 0.022,       // ~2.2% size
  },
  SizeTier {
    range: (1024, 10240),  // Medium Content
    item_pct: 0.20,        // 20% items
    size_pct: 0.12,        // ~12% size
  },
  SizeTier {
    range: (10240, 102400), // Large Objects
    item_pct: 0.04,         // 4% items
    size_pct: 0.24,         // ~24% size
  },
  SizeTier {
    range: (102400, 1048576), // Huge Blobs
    item_pct: 0.01,           // 1% items
    size_pct: 0.615,          // ~61.5% size
  },
];

/// Zipf data generator configuration / Zipf数据生成器配置
#[derive(Clone)]
pub struct ZipfDataConfig {
  /// Number of unique keys / 唯一键的数量
  pub num_keys: usize,
  /// Zipf exponent for key access frequency / 键访问频率的Zipf指数
  pub zipf_s: f64,
  /// Random seed / 随机种子
  pub seed: u64,
}

impl Default for ZipfDataConfig {
  fn default() -> Self {
    Self {
      num_keys: 1_000_000,
      zipf_s: 1.0,
      seed: 42,
    }
  }
}

/// Zipf sampler for generating indices / Zipf采样器，用于生成索引
struct ZipfSampler {
  n: usize,
  s: f64,
  harmonic: f64,
}

impl ZipfSampler {
  fn new(n: usize, s: f64) -> Self {
    // Cache harmonic number at initialization
    // 初始化时缓存调和数
    let harmonic = Self::calc_harmonic(n, s);
    Self { n, s, harmonic }
  }

  /// Sample an index using the rejection sampling method
  /// 使用拒绝采样方法采样索引
  #[inline]
  fn sample(&self, rng: &mut fastrand::Rng) -> usize {
    loop {
      let u = rng.f64();
      let v = rng.f64();
      let k = (u * self.n as f64) as usize;
      if k >= self.n {
        continue;
      }
      let zipf_prob = 1.0 / ((k + 1) as f64).powf(self.s);
      let threshold = zipf_prob * self.harmonic;
      if v < threshold {
        return k;
      }
    }
  }

  /// Calculate harmonic number H(n, s)
  /// 计算调和数 H(n, s)
  #[inline]
  fn calc_harmonic(n: usize, s: f64) -> f64 {
    (1..=n).map(|i| 1.0 / (i as f64).powf(s)).sum()
  }
}

/// Zipf data generator / Zipf数据生成器
pub struct ZipfDataGenerator {
  config: ZipfDataConfig,
  rng: fastrand::Rng,
  // Per-tier samplers for uniform Zipf across size tiers
  // 每层采样器，确保各大小层都有 Zipf 热点
  tier_samplers: Vec<ZipfSampler>,
  tier_ranges: Vec<(usize, usize)>, // (start_idx, end_idx) for each tier
}

impl ZipfDataGenerator {
  /// Create a new Zipf data generator / 创建新的Zipf数据生成器
  pub fn new(config: ZipfDataConfig) -> Self {
    let seed = config.seed;

    // Calculate tier counts / 计算每层数量
    let tier_counts = [
      (config.num_keys as f64 * SIZE_TIERS[0].item_pct).round() as usize,
      (config.num_keys as f64 * SIZE_TIERS[1].item_pct).round() as usize,
      (config.num_keys as f64 * SIZE_TIERS[2].item_pct).round() as usize,
      (config.num_keys as f64 * SIZE_TIERS[3].item_pct).round() as usize,
      (config.num_keys as f64 * SIZE_TIERS[4].item_pct).round() as usize,
    ];

    // Adjust last tier / 调整最后一层
    let mut tier_counts = tier_counts;
    let sum: usize = tier_counts.iter().sum();
    if sum != config.num_keys {
      tier_counts[4] = tier_counts[4].saturating_add(config.num_keys).saturating_sub(sum);
    }

    // Create per-tier samplers / 创建每层采样器
    let mut tier_samplers = Vec::with_capacity(5);
    let mut tier_ranges = Vec::with_capacity(5);
    let mut start_idx = 0;

    for &count in &tier_counts {
      if count > 0 {
        tier_samplers.push(ZipfSampler::new(count, config.zipf_s));
        tier_ranges.push((start_idx, start_idx + count));
        start_idx += count;
      } else {
        tier_samplers.push(ZipfSampler::new(1, config.zipf_s));
        tier_ranges.push((0, 0));
      }
    }

    Self {
      config,
      rng: fastrand::Rng::with_seed(seed),
      tier_samplers,
      tier_ranges,
    }
  }



  /// Generate a key-value pair with specified total size / 生成指定总大小的键值对
  #[inline]
  fn generate_kv(&mut self, total_size: usize) -> (Vec<u8>, Vec<u8>) {
    // Key is ~10% of total, value is ~90%
    // 键约占总大小的 10%，值约占 90%
    let key_size = (total_size / 10).max(8).min(256);
    let value_size = total_size.saturating_sub(key_size);

    let mut key = vec![0u8; key_size];
    let mut value = vec![0u8; value_size];

    // Fill with random data
    // 填充随机数据
    for byte in &mut key {
      *byte = self.rng.u8(..);
    }
    for byte in &mut value {
      *byte = self.rng.u8(..);
    }

    (key, value)
  }

  /// Generate all key-value pairs / 生成所有键值对
  /// Ensures exact distribution match (error < 0.1%) / 确保精确匹配分布（误差 < 0.1%）
  pub fn generate_all(&mut self) -> Vec<(Vec<u8>, Vec<u8>)> {
    let mut data = Vec::with_capacity(self.config.num_keys);

    // Calculate exact counts per tier / 计算每层的精确数量
    let tier_counts = [
      (self.config.num_keys as f64 * SIZE_TIERS[0].item_pct).round() as usize,
      (self.config.num_keys as f64 * SIZE_TIERS[1].item_pct).round() as usize,
      (self.config.num_keys as f64 * SIZE_TIERS[2].item_pct).round() as usize,
      (self.config.num_keys as f64 * SIZE_TIERS[3].item_pct).round() as usize,
      (self.config.num_keys as f64 * SIZE_TIERS[4].item_pct).round() as usize,
    ];

    // Adjust last tier to match exact total / 调整最后一层以匹配总数
    let mut tier_counts = tier_counts;
    let sum: usize = tier_counts.iter().sum();
    if sum != self.config.num_keys {
      tier_counts[4] = tier_counts[4].saturating_add(self.config.num_keys).saturating_sub(sum);
    }

    // First pass: estimate total size to calculate target / 第一遍：估算总大小以计算目标
    let mut estimated_total = 0u64;
    for (i, &count) in tier_counts.iter().enumerate() {
      let tier = &SIZE_TIERS[i];
      let avg_size = (tier.range.0 + tier.range.1) / 2;
      estimated_total += count as u64 * avg_size as u64;
    }

    // Calculate target size per tier based on size_pct / 基于 size_pct 计算每层目标大小
    let mut tier_target_sizes = [0u64; 5];
    for i in 0..5 {
      tier_target_sizes[i] = (estimated_total as f64 * SIZE_TIERS[i].size_pct) as u64;
    }

    let mut key_idx: usize = 0;

    // Generate items for each tier with precise size control / 为每层生成条目，精确控制大小
    for (tier_idx, &count) in tier_counts.iter().enumerate() {
      if count == 0 {
        continue;
      }

      let tier = &SIZE_TIERS[tier_idx];
      let target_total_size = tier_target_sizes[tier_idx];
      let target_avg_size = (target_total_size / count as u64) as usize;

      // Clamp to tier range / 限制在层级范围内
      let target_avg_size = target_avg_size.max(tier.range.0).min(tier.range.1);

      for item_idx in 0..count {
        // Use controlled variance to hit exact target / 使用受控方差以精确命中目标
        let total_size = if item_idx < count - 1 {
          // For most items, use small variance around target / 对大多数条目，使用目标附近的小方差
          let variance = ((tier.range.1 - tier.range.0) / 20).max(1);
          let min_size = target_avg_size.saturating_sub(variance).max(tier.range.0);
          let max_size = (target_avg_size + variance).min(tier.range.1);
          self.rng.usize(min_size..=max_size)
        } else {
          // Last item: adjust to hit exact target / 最后一项：调整以精确命中目标
          let start_idx = key_idx.saturating_sub(count - 1);
          let current_sum: u64 = data[start_idx..key_idx]
            .iter()
            .map(|(k, v): &(Vec<u8>, Vec<u8>)| (k.len() + v.len()) as u64)
            .sum();
          let remaining = target_total_size.saturating_sub(current_sum) as usize;
          remaining.max(tier.range.0).min(tier.range.1)
        };

        let (mut key, value) = self.generate_kv(total_size);

        // Add key index prefix to ensure uniqueness / 添加键索引前缀以确保唯一性
        let prefix = format!("k{key_idx:09x}");
        let prefix_bytes = prefix.as_bytes();
        let copy_len = prefix_bytes.len().min(key.len());
        key[..copy_len].copy_from_slice(&prefix_bytes[..copy_len]);

        data.push((key, value));
        key_idx += 1;
      }
    }

    data
  }

  /// Sample a key index based on access frequency Zipf distribution / 基于访问频率Zipf分布采样键索引
  /// Each size tier has its own Zipf distribution / 每个大小层都有自己的 Zipf 分布
  #[inline]
  pub fn sample_key_index(&mut self) -> usize {
    // First select tier uniformly / 首先均匀选择层级
    let tier_idx = self.rng.usize(0..5);
    let (start, end) = self.tier_ranges[tier_idx];
    
    if start >= end {
      // Empty tier, fallback to tier 0 / 空层级，回退到层级 0
      let offset = self.tier_samplers[0].sample(&mut self.rng);
      self.tier_ranges[0].0 + offset
    } else {
      // Sample within tier using Zipf / 在层级内使用 Zipf 采样
      let offset = self.tier_samplers[tier_idx].sample(&mut self.rng);
      start + offset
    }
  }

  /// Calculate total size of all data / 计算所有数据的总大小
  #[inline]
  pub fn total_size(data: &[(Vec<u8>, Vec<u8>)]) -> u64 {
    data.iter().map(|(k, v)| (k.len() + v.len()) as u64).sum()
  }

  /// Calculate average item size / 计算平均条目大小
  #[inline]
  pub fn avg_size(data: &[(Vec<u8>, Vec<u8>)]) -> usize {
    if data.is_empty() {
      return 0;
    }
    let total = Self::total_size(data);
    (total / data.len() as u64) as usize
  }

  /// Get min and max item sizes / 获取最小和最大条目大小
  pub fn size_range(data: &[(Vec<u8>, Vec<u8>)]) -> (usize, usize) {
    if data.is_empty() {
      return (0, 0);
    }
    // Use fold to avoid intermediate Vec allocation
    // 使用 fold 避免中间 Vec 分配
    data
      .iter()
      .map(|(k, v)| k.len() + v.len())
      .fold((usize::MAX, 0), |(min, max), size| {
        (min.min(size), max.max(size))
      })
  }
}

#[cfg(test)]
mod tests {

  #[test]
  fn test_data_distribution_detailed() {
    println!("\n=== Testing Data Distribution ===\n");

    let config = ZipfDataConfig {
      num_keys: 10_000,
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

    // Analyze tier distribution
    // 分析层级分布
    let mut tier_stats = vec![(0, 0u64); 5];
    let tier_ranges = [
      (16, 100, "16-100B (Tiny)"),
      (100, 1024, "100B-1KB (Small)"),
      (1024, 10240, "1-10KB (Medium)"),
      (10240, 102400, "10-100KB (Large)"),
      (102400, 1048576, "100KB-1MB (Huge)"),
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
      "{:<20} {:>10} {:>10} {:>12} {:>10}",
      "Tier", "Count", "Count%", "Size(MB)", "Size%"
    );
    println!("{}", "-".repeat(65));

    for (idx, &(count, size_bytes)) in tier_stats.iter().enumerate() {
      let count_pct = count as f64 / data.len() as f64 * 100.0;
      let size_pct = size_bytes as f64 / total_size as f64 * 100.0;
      let size_mb = size_bytes as f64 / (1024.0 * 1024.0);
      println!(
        "{:<20} {:>10} {:>9.1}% {:>11.2} {:>9.1}%",
        tier_ranges[idx].2, count, count_pct, size_mb, size_pct
      );
    }

    println!("\nExpected Distribution (from README):");
    println!("{:<20} {:>10} {:>12}", "Tier", "Items%", "Size%");
    println!("{}", "-".repeat(45));
    println!("{:<20} {:>10} {:>12}", "Tiny Metadata", "40%", "~0.3%");
    println!("{:<20} {:>10} {:>12}", "Small Structs", "35%", "~2.2%");
    println!("{:<20} {:>10} {:>12}", "Medium Content", "20%", "~12%");
    println!("{:<20} {:>10} {:>12}", "Large Objects", "4%", "~24%");
    println!("{:<20} {:>10} {:>12}", "Huge Blobs", "1%", "~61%");
    println!();

    // Verify distribution is reasonable
    // 验证分布合理
    let tier0_pct = tier_stats[0].0 as f64 / data.len() as f64;
    let tier1_pct = tier_stats[1].0 as f64 / data.len() as f64;
    let tier4_size_pct = tier_stats[4].1 as f64 / total_size as f64;

    assert!(
      tier0_pct > 0.30 && tier0_pct < 0.50,
      "Tier 0 count should be ~40%, got {:.1}%",
      tier0_pct * 100.0
    );
    assert!(
      tier1_pct > 0.25 && tier1_pct < 0.45,
      "Tier 1 count should be ~35%, got {:.1}%",
      tier1_pct * 100.0
    );
    assert!(
      tier4_size_pct > 0.50 && tier4_size_pct < 0.70,
      "Tier 4 size should be ~61%, got {:.1}%",
      tier4_size_pct * 100.0
    );
  }
}
