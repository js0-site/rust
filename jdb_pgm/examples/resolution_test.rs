use std::time::Instant;

fn main() {
  let mut min_diff = u128::MAX;
  let mut diffs = Vec::new();

  // Measure overhead/resolution
  for _ in 0..1000 {
    let start = Instant::now();
    // Do nothing
    let d = start.elapsed().as_nanos();
    if d > 0 {
      min_diff = min_diff.min(d);
      diffs.push(d);
    }
  }

  diffs.sort();
  diffs.dedup();

  println!("Minimum non-zero elapsed: {} ns", min_diff);
  println!(
    "Observed discrete steps (first 5 unique): {:?}",
    &diffs[..diffs.len().min(5)]
  );

  // Verify 24MHz theory (41.66ns ticks)
  let tick_ns = 1_000_000_000.0 / 24_000_000.0;
  println!("Assuming 24MHz clock, 1 tick = {:.2} ns", tick_ns);
}
