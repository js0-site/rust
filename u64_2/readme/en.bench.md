## u64_2 vs vb Benchmark

Comparing u64_2 (pair encoding) with vb (varint) using 100,000 integers (mixed distribution: 60% small, 30% medium, 10% large).

### Results

| Library | Encode (M/s) | Decode (M/s) |
|---------|--------------|--------------|
| u64_2 | 2727.6 | 2258.2 |
| vb | 199.5 | 288.3 |

### Environment

macOS 26.1 (arm64) · Apple M2 Max · 12 cores · 64.0GB · rustc 1.94.0-nightly (21ff67df1 2025-12-15)
