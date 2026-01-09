use fastrand::Rng;
use std::hint::black_box;
use std::time::Instant;
use u64_2::{decode, encode};
use vb::{d_li, e_li};

const N: usize = 100_000;

/// Generate test data: mixed distribution
/// 生成测试数据：混合分布
fn gen_data(rng: &mut Rng) -> Vec<u64> {
  (0..N)
    .map(|_| match rng.u8(0..10) {
      0..=5 => rng.u64(1..10_000),          // 60% small
      6..=8 => rng.u64(10_000..1_000_000),  // 30% medium
      _ => rng.u64(1_000_000..100_000_000), // 10% large
    })
    .collect()
}

/// Benchmark vb::e_li/d_li
fn bench_vb(data: &[u64]) -> (u64, u64) {
  // Warmup
  for _ in 0..3 {
    let encoded = e_li(data.iter().copied());
    let _ = black_box(d_li(&encoded));
  }

  let start = Instant::now();
  for _ in 0..10 {
    black_box(e_li(black_box(data).iter().copied()));
  }
  let encode_ns = start.elapsed().as_nanos() as u64 / 10;

  let encoded = e_li(data.iter().copied());

  let start = Instant::now();
  for _ in 0..10 {
    let _ = black_box(d_li(black_box(&encoded)));
  }
  let decode_ns = start.elapsed().as_nanos() as u64 / 10;

  (encode_ns, decode_ns)
}

/// Benchmark u64_2 encode/decode (pairs)
fn bench_u64_2(data: &[u64]) -> (u64, u64) {
  let pairs: Vec<_> = data.chunks(2).map(|c| (c[0], c.get(1).copied().unwrap_or(0))).collect();

  // Warmup
  for _ in 0..3 {
    let mut buf = [0u8; 17];
    for &(a, b) in &pairs {
      black_box(encode(a, b, &mut buf));
    }
  }

  let start = Instant::now();
  for _ in 0..10 {
    let mut buf = [0u8; 17];
    for &(a, b) in black_box(&pairs) {
      black_box(encode(a, b, &mut buf));
    }
  }
  let encode_ns = start.elapsed().as_nanos() as u64 / 10;

  let encoded: Vec<Vec<u8>> = pairs
    .iter()
    .map(|&(a, b)| {
      let mut buf = [0u8; 17];
      let n = encode(a, b, &mut buf);
      buf[..n].to_vec()
    })
    .collect();

  let start = Instant::now();
  for _ in 0..10 {
    for d in black_box(&encoded) {
      black_box(decode(d));
    }
  }
  let decode_ns = start.elapsed().as_nanos() as u64 / 10;

  (encode_ns, decode_ns)
}

fn main() {
  let mut rng = Rng::with_seed(42);
  let data = gen_data(&mut rng);

  let (vb_enc, vb_dec) = bench_vb(&data);
  let (u64_2_enc, u64_2_dec) = bench_u64_2(&data);

  let json = format!(
    r#"{{"data_count":{N},"results":[{{"lib":"u64_2","encode_ns":{u64_2_enc},"decode_ns":{u64_2_dec}}},{{"lib":"vb","encode_ns":{vb_enc},"decode_ns":{vb_dec}}}]}}"#
  );

  println!("{json}");
}
