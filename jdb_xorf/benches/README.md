# BinaryFuse Filter Benchmark Framework
# BinaryFuse 过滤器评测框架

## Evaluation Metrics
## 评测指标

### 1. Build Performance (构建性能)
- **Mean Time**: Average time to construct filter from keys
- **Throughput**: Keys per second during construction
- **平均时间**：从键构建过滤器的平均时间
- **吞吐量**：构建期间每秒处理键数

### 2. Contains Performance (查询性能)
- **Mean Time**: Average time to check if key exists
- **Throughput**: Queries per second
- **平均时间**：检查键是否存在的平均时间
- **吞吐量**：每秒查询数

### 3. False Positive Rate (假阳率)
- **FP Count**: Number of false positives in test set
- **FP Rate**: Percentage of false positives
- **假阳数**：测试集中假阳性数量
- **假阳率**：假阳性百分比

## Filter Types
## 过滤器类型

- **BinaryFuse8**: 8-bit fingerprints, ~1/256 false positive rate
- **BinaryFuse16**: 16-bit fingerprints, ~1/65536 false positive rate
- **BinaryFuse32**: 32-bit fingerprints, ~1/4B false positive rate

## Running Benchmarks
## 运行评测

### Full benchmark (all libraries)
### 完整评测（所有库）
```bash
./bench.sh
```

### Benchmark specific library
### 评测特定库
```bash
# Only jdb_xorf
cargo bench --bench bench --features bench-jdb

# Only xorf
cargo bench --bench bench --features bench-xorf
```

## Implementation
## 实现

The framework uses:
- **Trait-based design**: `FilterBench` trait for unified interface
- **Feature flags**: Selective compilation via `bench-jdb`, `bench-xorf`
- **Criterion**: Statistical benchmarking with detailed reports
- **No false negatives**: All filters guarantee zero false negatives

框架使用：
- **基于 trait 的设计**：`FilterBench` trait 提供统一接口
- **特性标志**：通过 `bench-jdb`、`bench-xorf` 选择性编译
- **Criterion**：统计评测并生成详细报告
- **零假阴性**：所有过滤器保证零假阴性
