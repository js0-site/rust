## Pgm-Index Benchmark

Performance comparison of Pgm-Index vs Binary Search with different epsilon values.

### Data Size: 1,000,000

| Algorithm | Epsilon | Mean Time | Std Dev | Throughput | Memory |
|-----------|---------|-----------|---------|------------|--------|
| jdb_pgm | 32 | 17.85ns | 58.01ns | 56.01M/s | 1.01 MB |
| jdb_pgm | 64 | 17.91ns | 56.67ns | 55.83M/s | 512.00 KB |
| pgm_index | 32 | 20.13ns | 54.58ns | 49.67M/s | 8.35 MB |
| pgm_index | 64 | 23.16ns | 66.31ns | 43.18M/s | 8.38 MB |
| pgm_index | 128 | 25.91ns | 62.66ns | 38.60M/s | 8.02 MB |
| jdb_pgm | 128 | 26.15ns | 96.65ns | 38.25M/s | 256.00 KB |
| HashMap | null | 39.99ns | 130.55ns | 25.00M/s | 40.00 MB |
| Binary Search | null | 40.89ns | 79.06ns | 24.46M/s | - |
| BTreeMap | null | 84.21ns | 99.32ns | 11.87M/s | 16.83 MB |

### Accuracy Comparison: jdb_pgm vs pgm_index

| Data Size | Epsilon | jdb_pgm (Max) | jdb_pgm (Avg) | pgm_index (Max) | pgm_index (Avg) |
|-----------|---------|---------------|---------------|-----------------|------------------|
| 1,000,000 | 128 | 128 | 46.80 | 1024 | 511.28 |
| 1,000,000 | 32 | 32 | 11.35 | 256 | 127.48 |
| 1,000,000 | 64 | 64 | 22.59 | 512 | 255.39 |
### Build Time Comparison: jdb_pgm vs pgm_index

| Data Size | Epsilon | jdb_pgm (Time) | pgm_index (Time) | Speedup |
|-----------|---------|---------------------|-----------------|---------|
| 1,000,000 | 128 | 1.28ms | 1.26ms | 0.98x |
| 1,000,000 | 32 | 1.28ms | 1.27ms | 0.99x |
| 1,000,000 | 64 | 1.28ms | 1.20ms | 0.94x |
### Configuration
Query Count: 1500000
Data Sizes: 10,000, 100,000, 1,000,000
Epsilon Values: 32, 64, 128



---

### Epsilon (ε) Explained

*Epsilon (ε) controls the accuracy-speed trade-off:*

*Mathematical definition: ε defines the maximum absolute error between the predicted position and the actual position in the data array. When calling `load(data, epsilon, ...)`, ε guarantees |pred - actual| ≤ ε, where positions are indices within the data array of length `data.len()`.*

*Example: For 1M elements with ε=32, if the actual key is at position 1000:*
- ε=32 predicts position between 968-1032, then checks up to 64 elements
- ε=128 predicts position between 872-1128, then checks up to 256 elements


### Notes
#### What is Pgm-Index?
Pgm-Index (Piecewise Geometric Model Index) is a learned index structure that approximates the distribution of keys with piecewise linear models.
It provides O(log ε) search time with guaranteed error bounds, where ε controls the trade-off between memory and speed.

#### Why Compare with Binary Search?
Binary search is the baseline for sorted array lookup. Pgm-Index aims to:
- Match or exceed binary search performance
- Reduce memory overhead compared to traditional indexes
- Provide better cache locality for large datasets

#### Environment
- OS: macOS 26.1 (arm64)
- CPU: Apple M2 Max
- Cores: 12
- Memory: 64.0GB
- Rust: rustc 1.94.0-nightly (8d670b93d 2025-12-31)

#### References
- [Pgm-Index Paper](https://doi.org/10.1145/3373718.3394764)
- [Official Pgm-Index Site](https://pgm.di.unipi.it/)
- [Learned Indexes](https://arxiv.org/abs/1712.01208)
