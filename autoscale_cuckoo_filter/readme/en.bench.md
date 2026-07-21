## Benchmark Results

### Test Environment

| Item   | Value                                       |
| ------ | ------------------------------------------- |
| OS     | macOS 26.1 (arm64)                          |
| CPU    | Apple M2 Max                                |
| Cores  | 12                                          |
| Memory | 64.0 GB                                     |
| Rust   | rustc 1.94.0-nightly (8d670b93d 2025-12-31) |

Test: 100000 items, capacity=200000

### What is FPP?

**FPP (False Positive Probability)** is the probability that a filter incorrectly reports an item as present when it was never added. Lower FPP means higher accuracy but requires more memory. A typical FPP of 1% means about 1 in 100 queries for non-existent items will incorrectly return "possibly exists".

### Performance Comparison

| Library                 | FPP   | Contains (M/s) | Add (M/s)    | Remove (M/s) | Memory (KB) |
| ----------------------- | ----- | -------------- | ------------ | ------------ | ----------- |
| autoscale_cuckoo_filter | 0.16% | 79.66 (1.00)   | 30.24 (1.00) | 47.45 (1.00) | 353.0       |
| cuckoo_filter           | 0.15% | 18.10 (0.23)   | 11.40 (0.38) | 17.87 (0.38) | 353.0       |
| cuckoofilter            | 0.27% | 21.76 (0.27)   | 20.40 (0.67) | 15.66 (0.33) | 1024.0      |

_Ratio in parentheses: relative to autoscale_cuckoo_filter (1.00 = baseline)_
