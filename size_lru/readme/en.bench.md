## LRU Cache Benchmark

Real-world data distribution, fixed memory budget, comparing hit rate and effective OPS.

### Results

| Library | Hit Rate | Effective OPS | Perf | Memory |
|---------|----------|---------------|------|--------|
| size_lru | 74.94% | 0.21M/s | 100% | 65676.0KB |
| moka | 70.74% | 0.17M/s | 81% | 65664.3KB |
| mini-moka | 70.45% | 0.17M/s | 80% | 66057.6KB |
| clru | 57.89% | 0.13M/s | 60% | 65655.7KB |
| lru | 57.85% | 0.13M/s | 60% | 65707.0KB |
| hashlink | 57.84% | 0.13M/s | 60% | 65534.6KB |
| schnellru | 57.79% | 0.13M/s | 60% | 65503.9KB |

### Configuration

Memory: 64.0MB · Zipf s=1 · R/W/D: 90/9/1% · Miss: 5% · Ops: 120M×3

### Size Distribution

| Range | Items | Size |
|-------|-------|------|
| 16-100B | 39.54% | 0.24% |
| 100B-1KB | 35.42% | 2.12% |
| 1-10KB | 20.04% | 12.14% |
| 10-100KB | 4.00% | 24.27% |
| 100KB-1MB | 1.00% | 61.23% |

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
- Rust: rustc 1.94.0-nightly (21ff67df1 2025-12-15)

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

---

## How to Build?

This library depends on the hardware-accelerated hash library `gxhash`.

`gxhash` uses different acceleration instructions on different hardware.

- Compiles directly on macOS and other `arm` chips
- On `x86_64`, compilation requires enabling modern CPU features `aes` and `sse2`, which are generally supported

You can configure this in your build script as follows:

```bash
if [[ "$(uname -m)" == "x86_64" ]]; then
  export RUSTFLAGS="$RUSTFLAGS -C target-feature=+aes,+sse2"
fi
```

If you are deploying to your own machines (not distributing to third parties), you can be more aggressive:

```bash
export RUSTFLAGS="-C target-cpu=native"
```