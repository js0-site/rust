# 性能测试 (Performance Benchmarks)

本目录包含了 `vb` 库的全面性能测试套件，用于评估和监控变长字节编码的性能表现。

## 📁 文件结构

```
benches/
├── vbyte_benchmarks.rs      # 基础功能性能测试
├── stress_benchmarks.rs     # 压力测试和极限场景测试
├── comparison_benchmarks.rs # VB 库性能分析测试
└── README.md               # 本文档
```

## 🚀 快速开始

### 运行所有性能测试

```bash
# 使用提供的脚本（推荐）
./bench.sh

# 或直接使用 cargo
cargo bench
```

### 运行特定测试套件

```bash
# 基础功能测试
./bench.sh basic
cargo bench --bench vbyte_benchmarks

# 压力测试
./bench.sh stress
cargo bench --bench stress_benchmarks

# 性能分析测试
./bench.sh comparison
cargo bench --bench comparison_benchmarks

# 差分编码测试（需要 diff 特性）
./bench.sh diff
cargo bench --bench vbyte_benchmarks --features diff
```

### 运行快速测试

```bash
# 运行关键性能指标的快速测试
./bench.sh quick
```

### 运行特定模式的测试

```bash
# 运行所有编码相关的测试
./bench.sh pattern:encode.*

# 运行所有解码相关的测试
./bench.sh pattern:decode.*

# 运行特定大小的测试
./bench.sh pattern:.*_small
```

## 📊 测试套件详情

### 1. 基础功能测试 (`vbyte_benchmarks.rs`)

#### 单个值操作

- **编码测试**: 测试不同大小数值的编码性能
  - `encode_single_small`: 小值 (< 128)
  - `encode_single_medium`: 中等值 (~16K)
  - `encode_single_large`: 大值 (u64::MAX)
  - `encode_single_various`: 各种大小的混合值

- **解码测试**: 测试解码性能和优化效果
  - `decode_single_*`: 对应编码的解码测试
  - `decode_offset_optimized`: 使用偏移量优化的解码

#### 列表操作

- **编码测试**: 不同规模列表的编码性能
  - `encode_list_small`: 100 个元素
  - `encode_list_medium`: 1, 000 个元素
  - `encode_list_large`: 10, 000 个元素
  - `encode_list_random`: 随机数据列表

- **解码测试**: 对应的解码性能测试

#### 往返测试

- **往返性能**: 编码后立即解码的完整流程性能
  - `roundtrip_small/medium/large`: 不同规模的往返测试

#### 内存分配模式

- **分配策略**: 测试不同内存分配策略的性能影响
  - `encode_with_capacity_hint`: 使用容量启发式
  - `encode_without_capacity_hint`: 不使用容量提示

#### 边界情况

- **边界值测试**: 极端情况的处理性能
  - 零值、最大值、空列表等

### 2. 压力测试 (`stress_benchmarks.rs`)

#### 吞吐量扩展性

- **扩展性测试**: 不同数据量下的吞吐量表现
  - 100 → 1, 000, 000 元素的性能扩展曲线

#### 内存压力测试

- **大数据集**: 测试大数据集下的性能和内存使用
  - 100K → 2M 元素的内存压力测试

#### 单值模式测试

- **缓存友好性**: 测试不同数据模式的缓存表现
  - 相同值重复编码
  - 顺序值编码
  - 随机值编码

#### 解码模式测试

- **解码模式**: 不同数据分布的解码性能
  - 小值、中值、大值、随机值的解码

#### 缓冲区重用

- **内存重用**: 缓冲区重用 vs 重新分配的性能对比

#### 并发访问

- **并发测试**: 多线程环境下的性能表现

#### 最坏情况场景

- **极限场景**: 最不利于性能的场景测试
  - 最大字节值（10 字节/值）
  - 交替大小值（分支预测最坏情况）

### 3. 性能分析测试 (`comparison_benchmarks.rs`)

#### 单值性能测试

- **编码性能**: 不同大小数值的编码性能分析
  - 小值、中值、大值的编码测试
  - 边界值编码测试

#### 列表性能测试

- **批量操作**: 大规模数据的编码/解码性能
  - 真实数据分布的列表处理
  - 不同数据规模的性能表现

#### 往返性能测试

- **完整流程**: 编码后立即解码的完整流程性能
  - 端到端性能分析
  - 数据完整性验证

#### 编码大小分析

- **压缩效率**: VB 编码的压缩效率分析
  - 不同数据分布的编码大小
  - 空间效率评估

#### 特化模式分析

- **优化效果**: 特定数据模式下的性能表现
  - 小值密集数据的优化效果
  - 混合数据的性能分析

#### 内存效率分析

- **内存使用**: 内存分配策略的效率分析
  - 容量预分配的效果
  - 内存使用模式优化

## 📈 测试报告

### HTML 报告

运行测试后，Criterion 会自动生成详细的 HTML 报告：

```bash
# 主报告
open target/criterion/report/index.html

# 特定测试的报告
open target/criterion/encode_single_small/report/index.html
```

### 报告内容

- **性能图表**: 包含误差棒的性能对比图
- **统计分析**: 详细的统计信息（均值、中位数、标准差等）
- **变化检测**: 与上次运行的性能变化对比
- **平台信息**: 测试环境的硬件和软件信息

## 🔧 自定义测试

### 添加新测试

1. 在相应的基准文件中添加新的测试函数
2. 使用 `criterion_group!` 宏注册测试
3. 使用 `criterion_main!` 宏设置主函数

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn my_custom_benchmark(c: &mut Criterion) {
    c.bench_function("my_test", |b| {
        b.iter(|| {
            // 你的测试代码
            black_box(your_function())
        })
    });
}

criterion_group!(my_benches, my_custom_benchmark);
criterion_main!(my_benches);
```

### 测试数据生成

使用提供的测试数据生成函数：

```rust
// 生成测试数据
let test_data = generate_test_data();           // 预定义的边界值
let sequential_data = generate_sequential_data(count, start, step);
let random_data = generate_random_data(count, max);
let stress_data = generate_stress_data(size);   // 压力测试数据
```

### 性能测试最佳实践

1. **使用 `black_box`**: 防止编译器优化掉测试代码
2. **控制变量**: 一次只测试一个变量
3. **多次运行**: 让 Criterion 自动确定合适的运行次数
4. **冷启动**: 考虑缓存预热的影响
5. **内存预分配**: 测试不同内存分配策略

## 📋 性能指标

### 关键指标

| 指标       | 描述                     | 目标              |
| ---------- | ------------------------ | ----------------- |
| 编码吞吐量 | 每秒编码的元素数         | > 100M elements/s |
| 解码吞吐量 | 每秒解码的元素数         | > 100M elements/s |
| 延迟       | 单个操作的延迟           | < 10ns            |
| 内存效率   | 编码大小与原始数据的比率 | < 原始大小        |
| 缓存命中率 | L1/L2/L3 缓存命中率      | > 90%             |

### 性能回归检测

Criterion 会自动检测性能回归：

- **警告阈值**: 5% 的性能下降会触发警告
- **错误阈值**: 10% 的性能下降会触发错误
- **置信区间**: 95% 置信度的统计显著性

## 🛠️ 故障排除

### 常见问题

1. **测试运行缓慢**

   ```bash
   # 使用快速模式
   ./bench.sh quick

   # 减少测试数据量
   cargo bench -- --sample-size 10
   ```

2. **内存不足**

   ```bash
   # 减少并发测试
   cargo bench -- --test-timeout 30
   ```

3. **报告生成失败**
   ```bash
   # 清理并重新运行
   cargo clean
   cargo bench
   ```

### 调试模式

```bash
# 启用详细输出
cargo bench -- --verbose

# 保存原始输出
cargo bench 2>&1 | tee benchmark.log
```

## 📝 持续集成

### GitHub Actions 示例

```yaml
name: Performance Tests
on: [push, pull_request]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run benchmarks
        run: cargo bench --bench vbyte_benchmarks
      - name: Upload benchmark results
        uses: actions/upload-artifact@v2
        with:
          name: benchmark-results
          path: target/criterion/
```

## 🤝 贡献

欢迎贡献新的性能测试或改进现有测试！请确保：

1. 新测试有清晰的文档说明
2. 遵循现有的命名约定
3. 包含适当的边界情况
4. 提供性能基准和目标

## 📞 支持

如果遇到问题或有建议，请：

1. 查看现有的 GitHub Issues
2. 创建新的 Issue 并提供详细信息
3. 包含系统信息、Rust 版本和测试输出
