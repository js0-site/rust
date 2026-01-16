[English](#en) | [中文](#zh)

---

<a id="en"></a>


# vb

**The fastest VByte encoding library in the Rust ecosystem.**

Encode at **430M integers/sec**, decode at **415M integers/sec** — 2.4x faster encoding and 1.2x faster decoding than alternatives.

![VByte Encoding Benchmark](https://raw.githubusercontent.com/js0-site/rust/refs/heads/main/vb/svg/en.svg)

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Usage](#usage)
  - [Basic Encoding](#basic-encoding)
  - [Differential Encoding](#differential-encoding)
- [API Reference](#api-reference)
- [Performance](#performance)
- [Design](#design)

## Features

- **Blazing Fast**: Hand-optimized with loop unrolling, bounds check elimination, and CLZ instructions
- **Variable Byte Encoding**: Compresses `u64` integers using 1-10 bytes based on magnitude
- **Differential Encoding**: Optimizes strictly increasing sequences (requires `diff` feature)
- **Zero-Copy Decoding**: Decode directly from byte slices with offset tracking
- **Minimal Dependencies**: Only `thiserror` for error handling

## Installation

```toml
[dependencies]
vb = "0.2"

# With differential encoding support
vb = { version = "0.2", features = ["diff"] }
```

## Usage

### Basic Encoding

```rust
use vb::{e_li, d_li};

let numbers = vec![0, 127, 128, 16383, 16384, 2097151];

// Encode
let encoded = e_li(numbers.iter().cloned());
println!("Compressed to {} bytes", encoded.len());

// Decode
let decoded = d_li(&encoded).unwrap();
assert_eq!(numbers, decoded);
```

### Differential Encoding

Ideal for sorted sequences like timestamps, IDs, or offsets.

```rust
use vb::{e_diff, d_diff};

let timestamps = vec![1000000, 1000005, 1000010, 1000042];

// Stores only deltas: [1000000, 5, 5, 32]
let encoded = e_diff(&timestamps);

let decoded = d_diff(&encoded).unwrap();
assert_eq!(timestamps, decoded);
```

## API Reference

| Function | Description |
|----------|-------------|
| `e(value, buf)` | Encode single `u64`, append to buffer |
| `d(bytes)` | Decode single `u64`, return `(value, bytes_consumed)` |
| `e_li(iter)` | Encode iterator of `u64` to `Vec<u8>` |
| `d_li(bytes)` | Decode bytes to `Vec<u64>` |
| `e_diff(slice)` | Encode increasing sequence with delta compression |
| `d_diff(bytes)` | Decode delta-compressed sequence |

## Performance

Benchmarked with 10,000 integers (60% small, 30% medium, 10% large):

| Library | Encode (M/s) | Decode (M/s) |
|---------|--------------|--------------|
| **vb** | **430** | **415** |
| leb128 | 289 | 213 |
| integer-encoding | 176 | 349 |

Run benchmarks yourself:

```bash
./bench.sh
```

## Design

VByte uses 7 bits per byte for data, with the MSB as continuation flag:

- `MSB = 0`: Final byte
- `MSB = 1`: More bytes follow

Key optimizations:

- **Fast path**: Single-byte values (< 128) skip all loops
- **Loop unrolling**: 2-5 byte cases fully unrolled
- **Bounds elimination**: Unsafe pointer arithmetic when ≥10 bytes available
- **CLZ instruction**: `leading_zeros()` calculates byte count in one CPU cycle

## Bench

## VByte Encoding Benchmark

Comparing varint encoding libraries with 10,000 integers (mixed distribution: 60% small, 30% medium, 10% large).

### Results

| Library | Encode (M/s) | Decode (M/s) |
|---------|--------------|--------------|
| vb | 430.5 | 414.9 |
| integer-encoding | 176.2 | 348.6 |
| leb128 | 289.2 | 212.9 |

### Environment

macOS 26.1 (arm64) · Apple M2 Max · 12 cores · 64.0GB · rustc 1.94.0-nightly (21ff67df1 2025-12-15)

---

## About

This project is an open-source component of [js0.site ⋅ Refactoring the Internet Plan](https://js0.site).

We are redefining the development paradigm of the Internet in a componentized way. Welcome to follow us:

* [Google Group](https://groups.google.com/g/js0-site)
* [js0site.bsky.social](https://bsky.app/profile/js0site.bsky.social)

---

<a id="zh"></a>

# vb

**Rust 生态中最快的 VByte 编码库。**

编码速度 **4.3 亿整数/秒**，解码速度 **4.15 亿整数/秒** — 编码比同类库快 2.4 倍，解码快 1.2 倍。

![VByte 编码性能评测](https://raw.githubusercontent.com/js0-site/rust/refs/heads/main/vb/svg/zh.svg)

## 目录

- [功能特性](#功能特性)
- [安装](#安装)
- [使用指南](#使用指南)
  - [基础编码](#基础编码)
  - [差分编码](#差分编码)
- [API 参考](#api-参考)
- [性能](#性能)
- [设计](#设计)

## 功能特性

- **极致性能**：手工优化，包括循环展开、边界检查消除、CLZ 指令加速
- **变长字节编码**：根据数值大小，用 1-10 字节压缩 `u64` 整数
- **差分编码**：优化严格递增序列（需开启 `diff` 特性）
- **零拷贝解码**：直接从字节切片解码，支持偏移量追踪
- **依赖精简**：仅依赖 `thiserror` 处理错误

## 安装

```toml
[dependencies]
vb = "0.2"

# 启用差分编码
vb = { version = "0.2", features = ["diff"] }
```

## 使用指南

### 基础编码

```rust
use vb::{e_li, d_li};

let numbers = vec![0, 127, 128, 16383, 16384, 2097151];

// 编码
let encoded = e_li(numbers.iter().cloned());
println!("压缩至 {} 字节", encoded.len());

// 解码
let decoded = d_li(&encoded).unwrap();
assert_eq!(numbers, decoded);
```

### 差分编码

适用于时间戳、ID、偏移量等有序序列。

```rust
use vb::{e_diff, d_diff};

let timestamps = vec![1000000, 1000005, 1000010, 1000042];

// 仅存储差值: [1000000, 5, 5, 32]
let encoded = e_diff(&timestamps);

let decoded = d_diff(&encoded).unwrap();
assert_eq!(timestamps, decoded);
```

## API 参考

| 函数 | 说明 |
|------|------|
| `e(value, buf)` | 编码单个 `u64`，追加到缓冲区 |
| `d(bytes)` | 解码单个 `u64`，返回 `(值, 消耗字节数)` |
| `e_li(iter)` | 将 `u64` 迭代器编码为 `Vec<u8>` |
| `d_li(bytes)` | 将字节解码为 `Vec<u64>` |
| `e_diff(slice)` | 差分压缩递增序列 |
| `d_diff(bytes)` | 解码差分压缩序列 |

## 性能

测试数据：10,000 个整数（60% 小值，30% 中值，10% 大值）

| 库 | 编码 (百万/秒) | 解码 (百万/秒) |
|----|----------------|----------------|
| **vb** | **430** | **415** |
| leb128 | 289 | 213 |
| integer-encoding | 176 | 349 |

运行评测：

```bash
./bench.sh
```

## 设计

VByte 每字节用 7 位存数据，最高位 (MSB) 作为延续标志：

- `MSB = 0`：最后一个字节
- `MSB = 1`：后续还有字节

核心优化：

- **快速路径**：单字节值 (< 128) 跳过所有循环
- **循环展开**：2-5 字节场景完全展开
- **边界消除**：剩余 ≥10 字节时使用 unsafe 指针运算
- **CLZ 指令**：`leading_zeros()` 单周期计算所需字节数

## 评测

## VByte 编码性能评测

对比 varint 编码库，测试数据：10,000 个整数（混合分布：60% 小值，30% 中值，10% 大值）。

### 结果

| 库 | 编码 (百万/秒) | 解码 (百万/秒) |
|----|----------------|----------------|
| vb | 430.5 | 414.9 |
| integer-encoding | 176.2 | 348.6 |
| leb128 | 289.2 | 212.9 |

### 环境

macOS 26.1 (arm64) · Apple M2 Max · 12 核 · 64.0GB · rustc 1.94.0-nightly (21ff67df1 2025-12-15)

---

## 关于

本项目为 [js0.site ⋅ 重构互联网计划](https://js0.site) 的开源组件。

我们正在以组件化的方式重新定义互联网的开发范式，欢迎关注：

* [谷歌邮件列表](https://groups.google.com/g/js0-site)
* [js0site.bsky.social](https://bsky.app/profile/js0site.bsky.social)
