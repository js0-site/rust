## VByte Encoding Benchmark

Comparing varint encoding libraries with 10,000 integers (mixed distribution: 60% small, 30% medium, 10% large).

### Results

| Library          | Encode (M/s) | Decode (M/s) |
| ---------------- | ------------ | ------------ |
| vb               | 430.5        | 414.9        |
| integer-encoding | 176.2        | 348.6        |
| leb128           | 289.2        | 212.9        |

### Environment

macOS 26.1 (arm64) · Apple M2 Max · 12 cores · 64.0GB · rustc 1.94.0-nightly (21ff67df1 2025-12-15)
