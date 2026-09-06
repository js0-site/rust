## Benchmark Results

### Test Environment

| Item | Value |
|------|-------|
| OS | macOS 26.5.1 (arm64) |
| CPU | Apple M2 Max |
| Cores | 12 |
| Memory | 64.0 GB |
| Rust | rustc 1.98.0 (88d9e12ae 2026-08-18) |

Test: 100000 items, capacity=200000

### What is FPP?

**FPP (False Positive Probability)** is the probability that a filter incorrectly reports an item as present when it was never added. Lower FPP means higher accuracy but requires more memory. A typical FPP of 1% means about 1 in 100 queries for non-existent items will incorrectly return "possibly exists".

### Performance Comparison

| Library | FPP | Contains (M/s) | Add (M/s) | Remove (M/s) | Memory (KB) |
|---------|-----|----------------|-----------|--------------|-------------|
| autoscale_cuckoo_filter | 0.17% | 84.60 (1.00) | 24.76 (1.00) | 52.32 (1.00) | 353.0 |
| cuckoo_filter | 0.15% | 18.06 (0.21) | 10.90 (0.44) | 18.78 (0.36) | 353.0 |
| cuckoofilter | 0.27% | 23.84 (0.28) | 23.71 (0.96) | 19.05 (0.36) | 1024.0 |

*Ratio in parentheses: relative to autoscale_cuckoo_filter (1.00 = baseline)*