## LRU Cache Benchmark

Real-world data distribution, fixed memory budget, comparing hit rate and effective OPS.

### Results

| Library | Hit Rate | Effective OPS | Perf | Memory |
|---------|----------|---------------|------|--------|
| size_lru | 74.86% | 0.17M/s | 100% | 63.751MB |
| schnellru | 44.16% | 0.09M/s | 54% | 64.043MB |
| lru | 43.49% | 0.09M/s | 53% | 64.150MB |
| hashlink | 42.47% | 0.09M/s | 52% | 66.526MB |
| clru | 41.44% | 0.09M/s | 52% | 64.403MB |
| moka | 60.14% | 0.08M/s | 48% | 64.060MB |
| mini-moka | 58.18% | 0.08M/s | 46% | 63.989MB |

### Configuration

Memory: 64.0MB · Zipf s=1 · R/W/D: 90/9/1% · Miss: 5% · Ops: 0M×0

### Size Distribution

| Range | Items | Size |
|-------|-------|------|
| <100B | 40.00% | 0.30% |
| 100B-1KB | 35.00% | 2.20% |
| 1-10KB | 20.00% | 11.98% |
| 10-100KB | 4.00% | 24.01% |
| >=100KB | 1.00% | 61.51% |

---

### Notes

#### Data Distribution

Based on Facebook USR/APP/VAR pools and Twitter/Meta traces:

| Tier | Size | Items% | Size% |
|------|------|--------|-------|
| Tiny Metadata | 16-100B | 40% | ~0.3% |
| Small Structs | 100B-1KB | 35% | ~2.2% |
| Medium Content | 1-10KB | 20% | ~12% |
| Large Objects | 10-100KB | 4% | ~24% |
| Huge Blobs | 100KB-1MB | 1% | ~61% |

#### Operation Mix

| Op | % | Source |
|----|---|--------|
| Read | 90% | Twitter: 99%+ reads, TAO: 99.8% reads |
| Write | 9% | TAO: ~0.1% writes, relaxed for testing |
| Delete | 1% | TAO: ~0.1% deletes |

#### Environment

- OS: macOS 26.1 (arm64)
- CPU: Apple M2 Max
- Cores: 12
- Memory: 64.0GB
- Rust: rustc 1.94.0-nightly (8d670b93d 2025-12-31)

#### Why Effective OPS?

Raw OPS ignores hit rate — a cache with 99% hit rate at 1M ops/s outperforms one with 50% hit rate at 2M ops/s in real workloads.

**Effective OPS** models real-world performance by penalizing cache misses with actual I/O latency.


#### Why NVMe Latency?

LRU caches typically sit in front of persistent storage (databases, KV stores). On cache miss, data must be fetched from disk.

Miss penalty: 18,000ns — measured via DapuStor X5900 PCIe 5.0 NVMe (18µs)


Formula: `effective_ops = 1 / (hit_time + miss_rate × miss_latency)`

- hit_time = 1 / raw_ops

- Higher hit rate → fewer disk reads → better effective throughput

#### References

- [cache_dataset](https://github.com/cacheMon/cache_dataset)
- OSDI'20: Twitter cache analysis
- FAST'20: Facebook RocksDB workloads
- ATC'13: Scaling Memcache at Facebook