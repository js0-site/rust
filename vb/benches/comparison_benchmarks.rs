use std::{
  fs,
  hint::black_box,
  io::Cursor,
  time::{Duration, Instant},
};

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use fastrand::Rng;
use integer_encoding::VarInt;
use leb128::{read, write};
use vb::{d_li, e_li};

fn fast_config() -> Criterion {
  Criterion::default()
    .warm_up_time(Duration::from_millis(500))
    .measurement_time(Duration::from_secs(3))
}

fn generate_test_data() -> Vec<u64> {
  let mut rng = Rng::with_seed(42);
  (0..10000)
    .map(|_| {
      // Realistic distribution:
      // 60% small (<128), 30% medium (<2^21), 10% large
      match rng.u8(0..10) {
        0..=5 => rng.u64(0..128),
        6..=8 => rng.u64(0..(1 << 21)),
        _ => rng.u64(..),
      }
    })
    .collect()
}

// ============ Encode helpers ============

fn encode_vb(data: &[u64]) -> Vec<u8> {
  e_li(data.iter().cloned())
}

fn encode_integer_encoding(data: &[u64]) -> Vec<u8> {
  let mut buf = Vec::with_capacity(data.len() * 2);
  for &v in data {
    let mut tmp = [0u8; 10];
    let len = v.encode_var(&mut tmp);
    buf.extend_from_slice(&tmp[..len]);
  }
  buf
}

fn encode_leb128(data: &[u64]) -> Vec<u8> {
  let mut buf = Vec::with_capacity(data.len() * 2);
  for &v in data {
    write::unsigned(&mut buf, v).unwrap();
  }
  buf
}

// ============ Decode helpers ============

fn decode_vb(encoded: &[u8]) -> Vec<u64> {
  d_li(encoded).unwrap()
}

fn decode_integer_encoding(encoded: &[u8]) -> Vec<u64> {
  let mut result = Vec::with_capacity(encoded.len() / 2);
  let mut offset = 0;
  while offset < encoded.len() {
    let (val, len) = u64::decode_var(&encoded[offset..]).unwrap();
    result.push(val);
    offset += len;
  }
  result
}

fn decode_leb128(encoded: &[u8]) -> Vec<u64> {
  let mut result = Vec::with_capacity(encoded.len() / 2);
  let mut cursor = Cursor::new(encoded);
  while (cursor.position() as usize) < encoded.len() {
    let val = read::unsigned(&mut cursor).unwrap();
    result.push(val);
  }
  result
}

// ============ Benchmarks ============

fn bench_encode(c: &mut Criterion) {
  let mut group = c.benchmark_group("e_li");
  let data = generate_test_data();
  group.throughput(Throughput::Elements(data.len() as u64));

  group.bench_with_input(BenchmarkId::new("vb", "10k"), &data, |b, data| {
    b.iter(|| black_box(encode_vb(black_box(data))))
  });

  group.bench_with_input(
    BenchmarkId::new("integer-encoding", "10k"),
    &data,
    |b, data| b.iter(|| black_box(encode_integer_encoding(black_box(data)))),
  );

  group.bench_with_input(BenchmarkId::new("leb128", "10k"), &data, |b, data| {
    b.iter(|| black_box(encode_leb128(black_box(data))))
  });

  group.finish();
}

fn bench_decode(c: &mut Criterion) {
  let mut group = c.benchmark_group("d_li");
  let data = generate_test_data();

  let encoded_vb = encode_vb(&data);
  let encoded_ie = encode_integer_encoding(&data);
  let encoded_leb = encode_leb128(&data);

  group.throughput(Throughput::Elements(data.len() as u64));

  group.bench_with_input(BenchmarkId::new("vb", "10k"), &encoded_vb, |b, enc| {
    b.iter(|| black_box(decode_vb(black_box(enc))))
  });

  group.bench_with_input(
    BenchmarkId::new("integer-encoding", "10k"),
    &encoded_ie,
    |b, enc| b.iter(|| black_box(decode_integer_encoding(black_box(enc)))),
  );

  group.bench_with_input(BenchmarkId::new("leb128", "10k"), &encoded_leb, |b, enc| {
    b.iter(|| black_box(decode_leb128(black_box(enc))))
  });

  group.finish();
}

// Output JSON for svg.js and table.js
fn output_json(c: &mut Criterion) {
  let data = generate_test_data();

  let encoded_vb = encode_vb(&data);
  let encoded_ie = encode_integer_encoding(&data);
  let encoded_leb = encode_leb128(&data);

  // Warm up
  for _ in 0..100 {
    let _ = encode_vb(&data);
    let _ = decode_vb(&encoded_vb);
    let _ = encode_integer_encoding(&data);
    let _ = decode_integer_encoding(&encoded_ie);
    let _ = encode_leb128(&data);
    let _ = decode_leb128(&encoded_leb);
  }

  let iterations = 1000;

  // Measure vb
  let start = Instant::now();
  for _ in 0..iterations {
    let _ = black_box(encode_vb(black_box(&data)));
  }
  let vb_encode_ns = start.elapsed().as_nanos() as f64 / iterations as f64;

  let start = Instant::now();
  for _ in 0..iterations {
    let _ = black_box(decode_vb(black_box(&encoded_vb)));
  }
  let vb_decode_ns = start.elapsed().as_nanos() as f64 / iterations as f64;

  // Measure integer-encoding
  let start = Instant::now();
  for _ in 0..iterations {
    let _ = black_box(encode_integer_encoding(black_box(&data)));
  }
  let ie_encode_ns = start.elapsed().as_nanos() as f64 / iterations as f64;

  let start = Instant::now();
  for _ in 0..iterations {
    let _ = black_box(decode_integer_encoding(black_box(&encoded_ie)));
  }
  let ie_decode_ns = start.elapsed().as_nanos() as f64 / iterations as f64;

  // Measure leb128
  let start = Instant::now();
  for _ in 0..iterations {
    let _ = black_box(encode_leb128(black_box(&data)));
  }
  let leb_encode_ns = start.elapsed().as_nanos() as f64 / iterations as f64;

  let start = Instant::now();
  for _ in 0..iterations {
    let _ = black_box(decode_leb128(black_box(&encoded_leb)));
  }
  let leb_decode_ns = start.elapsed().as_nanos() as f64 / iterations as f64;

  let json = serde_json::json!({
    "data_count": data.len(),
    "results": [
      {
        "lib": "vb",
        "encode_ns": vb_encode_ns,
        "decode_ns": vb_decode_ns,
        "encoded_bytes": encoded_vb.len(),
      },
      {
        "lib": "integer-encoding",
        "encode_ns": ie_encode_ns,
        "decode_ns": ie_decode_ns,
        "encoded_bytes": encoded_ie.len(),
      },
      {
        "lib": "leb128",
        "encode_ns": leb_encode_ns,
        "decode_ns": leb_decode_ns,
        "encoded_bytes": encoded_leb.len(),
      },
    ]
  });

  fs::write("bench.json", serde_json::to_string_pretty(&json).unwrap()).unwrap();

  // Dummy benchmark to satisfy criterion
  c.bench_function("_output_json", |b| b.iter(|| 1));
}

criterion_group! {
  name = benches;
  config = fast_config();
  targets = bench_encode, bench_decode, output_json,
}

criterion_main!(benches);
