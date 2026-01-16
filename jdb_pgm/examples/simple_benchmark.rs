//! Simple benchmark demonstrating Pgm-Index performance vs binary search
//! 简单基准测试，展示 Pgm 索引与二分查找的性能对比

use std::time::Instant;

use jdb_pgm::PgmIndex;
use rand::{Rng, SeedableRng, rngs::StdRng};

fn main() {
  println!("=== Pgm-Index Simple Benchmark ===\n");

  const N: usize = 10_000_000;
  const QUERY_COUNT: usize = 100_000;
  let mut rng = StdRng::seed_from_u64(42);

  println!("Creating dataset with {N} elements...");
  let data: Vec<u64> = (0..N as u64).collect();

  let mut queries = Vec::with_capacity(QUERY_COUNT);
  for _ in 0..QUERY_COUNT {
    queries.push(rng.random_range(0..N as u64));
  }

  for &epsilon in &[16usize, 32, 64, 128] {
    test_epsilon(epsilon, &data, &queries);
  }

  println!(
    "\nTip: smaller ε ⇒ more segments (faster queries, higher memory); \
         larger ε ⇒ fewer segments (slower, lower memory)."
  );
}

fn test_epsilon(epsilon: usize, data: &[u64], queries: &[u64]) {
  println!("=== Pgm-Index (ε = {epsilon}) ===");

  let build_start = Instant::now();
  let index = PgmIndex::load(data.to_vec(), epsilon, true).unwrap();
  let build_time = build_start.elapsed();

  println!("Build time: {build_time:?}");
  println!("Segments: {}", index.segment_count());
  println!("Avg segment size: {:.1}", index.avg_segment_size());
  println!(
    "Memory usage: {:.2} MB",
    index.memory_usage() as f64 / 1024.0 / 1024.0
  );
  println!(
    "Memory overhead: {:.2}%",
    (index.memory_usage() as f64 / (data.len() * 8) as f64 - 1.0) * 100.0
  );

  // Single query smoke (10)
  // 单次查询测试 (10 次)
  let single_start = Instant::now();
  let mut hits = 0usize;
  for &q in &queries[0..queries.len().min(10)] {
    if index.get(q).is_some() {
      hits += 1;
    }
  }
  let single_time = single_start.elapsed();
  println!("Single query time ({hits} hits / 10): {single_time:?}");

  // Batch throughput
  // 批量吞吐量
  let batch_start = Instant::now();
  let mut hits = 0usize;
  for &q in queries {
    if index.get(q).is_some() {
      hits += 1;
    }
  }
  let batch_time = batch_start.elapsed();
  let ns_per_query = batch_time.as_nanos() as f64 / (queries.len() as f64);
  println!("Batch query time: {batch_time:?}");
  println!(
    "Batch throughput: {:.0} queries/sec",
    (queries.len() as f64) / batch_time.as_secs_f64()
  );
  println!("Batch average: {ns_per_query:.1} ns/query");
  println!("Hits: {hits}/{}", queries.len());

  // Edge keys
  // 边界键测试
  let test_keys = vec![data[0], data[data.len() / 2], data[data.len() - 1]];
  let start = Instant::now();
  for &key in &test_keys {
    let _ = index.get(key);
  }
  let query_time = start.elapsed();
  println!("Query time (3 edge keys): {query_time:?}\n");
}
