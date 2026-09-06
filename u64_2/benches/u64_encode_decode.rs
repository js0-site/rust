use std::{hint::black_box, time::Duration};

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use fastrand::Rng;
use u64_2::{decode, encode};

const N: usize = 10000;

/// 生成模拟文件长度和offset数据
/// 模拟真实场景: 文件长度通常 1B~100MB, offset 可能很大
fn gen_file_data(rng: &mut Rng) -> Vec<(u64, u64)> {
  (0..N)
    .map(|_| {
      // 文件长度: 1B ~ 100MB (大部分小文件)
      // file size: 1B ~ 100MB (mostly small files)
      let len = match rng.u8(0..10) {
        0..=5 => rng.u64(1..10_000),          // 60% 小文件 <10KB
        6..=8 => rng.u64(10_000..1_000_000),  // 30% 中等 10KB~1MB
        _ => rng.u64(1_000_000..100_000_000), // 10% 大文件 1MB~100MB
      };
      // offset: 累积偏移，模拟文件在存储中的位置
      // offset: cumulative, simulating file position in storage
      let offset = rng.u64(0..u64::MAX / 2);
      (len, offset)
    })
    .collect()
}

fn bench_file_storage(c: &mut Criterion) {
  let mut rng = Rng::with_seed(42);
  let data = gen_file_data(&mut rng);

  let mut group = c.benchmark_group("file_storage");
  group.warm_up_time(Duration::from_secs(1));
  group.throughput(Throughput::Elements(N as u64));

  // 编码评测 / encode benchmark
  group.bench_function("encode", |b| {
    b.iter(|| {
      let mut buf = [0u8; 17];
      for &(len, offset) in &data {
        encode(black_box(len), black_box(offset), black_box(&mut buf));
      }
    })
  });

  // 预编码数据 / pre-encode data
  let encoded: Vec<Vec<u8>> = data
    .iter()
    .map(|&(len, offset)| {
      let mut buf = [0u8; 17];
      let n = encode(len, offset, &mut buf);
      buf[..n].to_vec()
    })
    .collect();

  // 解码评测 / decode benchmark
  group.bench_function("decode", |b| {
    b.iter(|| {
      for d in &encoded {
        decode(black_box(d));
      }
    })
  });

  // 编解码往返 / roundtrip
  group.bench_function("roundtrip", |b| {
    b.iter(|| {
      let mut buf = [0u8; 17];
      for &(len, offset) in &data {
        let n = encode(black_box(len), black_box(offset), black_box(&mut buf));
        decode(black_box(&buf[..n]));
      }
    })
  });

  group.finish();
}

criterion_group!(benches, bench_file_storage);
criterion_main!(benches);
