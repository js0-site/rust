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

| 函数            | 说明                                    |
| --------------- | --------------------------------------- |
| `e(value, buf)` | 编码单个 `u64`，追加到缓冲区            |
| `d(bytes)`      | 解码单个 `u64`，返回 `(值, 消耗字节数)` |
| `e_li(iter)`    | 将 `u64` 迭代器编码为 `Vec<u8>`         |
| `d_li(bytes)`   | 将字节解码为 `Vec<u64>`                 |
| `e_diff(slice)` | 差分压缩递增序列                        |
| `d_diff(bytes)` | 解码差分压缩序列                        |

## 性能

测试数据：10,000 个整数（60% 小值，30% 中值，10% 大值）

| 库               | 编码 (百万/秒) | 解码 (百万/秒) |
| ---------------- | -------------- | -------------- |
| **vb**           | **430**        | **415**        |
| leb128           | 289            | 213            |
| integer-encoding | 176            | 349            |

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
