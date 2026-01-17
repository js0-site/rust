# 快速基准测试配置 / Quick Benchmark Configuration

## 当前配置（约 3 秒）

```rust
group.measurement_time(Duration::from_secs(2));
group.warm_up_time(Duration::from_millis(500));
group.sample_size(50);
num_keys: 50_000
```

## 更快配置选项

### 超快模式（约 1 秒）
```rust
group.measurement_time(Duration::from_secs(1));
group.warm_up_time(Duration::from_millis(200));
group.sample_size(20);
num_keys: 10_000
```

### 开发模式（约 0.5 秒）
```rust
group.measurement_time(Duration::from_millis(500));
group.warm_up_time(Duration::from_millis(100));
group.sample_size(10);
num_keys: 5_000
```

### 完整模式（约 30 秒 - 用于发布）
```rust
group.measurement_time(Duration::from_secs(5));
group.warm_up_time(Duration::from_secs(2));
group.sample_size(100);
num_keys: 100_000
```

## 当前测试结果

- **size_lru**: 5.26 ms per 1000 ops = **190 K ops/s**
- **命中率**: 58.5%
- **测试时间**: ~3 秒 ⚡

## 使用建议

- **日常开发**: 使用当前配置（3秒）
- **快速验证**: 使用超快模式（1秒）
- **CI/CD**: 使用当前配置
- **发布前**: 使用完整模式（30秒）
