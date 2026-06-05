[English](#en) | [中文](#zh)

---

<a id="en"></a>

# CIter: Zero-Copy Circular Iterator for Rust

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Usage Examples](#usage-examples)
- [API Reference](#api-reference)
- [Design Philosophy](#design-philosophy)
- [Technical Stack](#technical-stack)
- [Project Structure](#project-structure)
- [Historical Context](#historical-context)

## Overview

CIter is a lightweight, zero-copy circular iterator library for Rust that enables efficient traversal of slices from any starting position. The iterator wraps around the slice boundaries, providing seamless circular access to data with index information without memory allocation or copying.

## Features

- **Zero-Copy Design**: References original slice data without allocation
- **Flexible Starting Position**: Begin iteration from any index within the slice
- **Automatic Wraparound**: Seamlessly continues from the beginning after reaching the end
- **Position Tracking**: Built-in position tracking for current iterator state
- **Index Information**: Returns both index and value for each element
- **Random Start Support**: Optional random starting position with `rand` feature
- **Memory Safe**: Leverages Rust's ownership system for safe memory access
- **Generic Implementation**: Works with any slice type `&[T]`

## Usage Examples

### Basic Circular Iteration

```rust
use citer::CIter;

let data = [1, 2, 3, 4, 5];
let iter = CIter::new(&data, 2); // Start from index 2
let result: Vec<_> = iter.collect();
// Result: [(2, &3), (3, &4), (4, &5), (0, &1), (1, &2)]
```

### Position Tracking

```rust
let data = [10, 20, 30];
let iter = CIter::new(&data, 2);
println!("Current position: {}", iter.pos()); // Output: 1
```

### Random Starting Position

```rust
// Requires "rand" feature
let data = [1, 2, 3, 4, 5];
let iter = CIter::rand(&data);
let result: Vec<_> = iter.collect();
// Result: Random permutation starting from random position with indices
```

### Manual Iteration

```rust
let data = [1, 2, 3];
let mut iter = CIter::new(&data, 1);

assert_eq!(iter.next(), Some((1, &2)));
assert_eq!(iter.next(), Some((2, &3)));
assert_eq!(iter.next(), Some((0, &1)));
assert_eq!(iter.next(), None); // Iterator exhausted
```

### Extracting Values Only

```rust
let data = [1, 2, 3, 4, 5];
let iter = CIter::new(&data, 2);
let values: Vec<&i32> = iter.map(|(_, &value)| value).collect();
// Result: [3, 4, 5, 1, 2]
```

## API Reference

### `CIter<'a, T>`

Main circular iterator struct with lifetime parameter `'a` and generic type `T`.

#### Fields

- `idx: usize` - Current index position
- `li: &'a [T]` - Reference to the slice data
- `ed: usize` - Number of elements already visited

#### Methods

- `new(li: &'a [T], pos: usize) -> Self`
  - Creates new circular iterator starting from specified position
  - Position is used as-is without bounds checking (handled by modulo in iteration)

- `pos(&self) -> usize`
  - Returns current logical position (0-based)
  - Returns `idx - 1` for non-zero indices, `0` for zero index

- `rand(li: &'a [T]) -> Self` (requires `rand` feature)
  - Creates iterator with random starting position
  - Uses thread-local random number generator

#### Iterator Implementation

Implements standard `Iterator` trait:

- `type Item = (usize, &'a T)` - Returns tuple of (index, reference to value)
- `fn next(&mut self) -> Option<Self::Item>`

## Design Philosophy

The library follows a minimalist approach focusing on performance and safety:

```mermaid
graph TD
  A[Slice Input] --> B[CIter::new]
  B --> C[Position Calculation]
  C --> D[Iterator Creation]
  D --> E[next Call]
  E --> F{Elements Visited < Length}
  F -->|Yes| G[Calculate Index with Modulo]
  G --> H[Return (index, value) Tuple]
  F -->|No| I[Return None]
  H --> J[Increment Counters]
  J --> E
  I --> K[Iterator Exhausted]
```

### Core Design Principles

1. **Zero-Copy Architecture**: References original data without duplication
2. **Bounded Iteration**: Guarantees termination after visiting all elements once
3. **Modulo Arithmetic**: Handles position wraparound efficiently
4. **Index Preservation**: Maintains original index information throughout iteration
5. **Lifetime Safety**: Ensures iterator cannot outlive referenced data

### Module Interaction Flow

- `CIter::new()` initializes iterator state
- `Iterator::next()` implements core circular logic with modulo arithmetic
- Position tracking maintains current state without additional allocations
- Optional `rand` feature provides randomized starting positions
- Index information is preserved in the returned tuples

## Technical Stack

- **Language**: Rust 2024 Edition
- **Core Dependencies**:
  - `aok` (0.1.18) - Result handling utilities
- **Optional Dependencies**:
  - `rand` (0.9.2) - Random starting position support
- **Development Dependencies**:
  - `aok` (0.1.18) - Test result handling
  - `log` (0.4.29) - Logging infrastructure
  - `loginit` (0.1.18) - Log initialization
  - `static_init` (1.0.4) - Static initialization
  - `log` (0.1.43) - Structured logging
  - `log_init` (0.1.34) - Local log initialization utilities

## Project Structure

```
citer/
├── src/
│   └── lib.rs          # Core CIter implementation
├── tests/
│   └── main.rs         # Integration tests
├── readme/
│   ├── en.md          # English documentation
│   └── zh.md          # Chinese documentation
├── Cargo.toml         # Project configuration
└── test.sh           # Test execution script
```

### Key Components

- **`CIter` struct**: Main iterator implementation with position and element tracking
- **Iterator trait**: Standard Rust iterator interface returning (index, value) tuples
- **Feature gates**: Optional functionality behind compile-time flags
- **Comprehensive tests**: Unit and integration test coverage

## Historical Context

Circular iterators have deep roots in computer science, log back to early work on circular buffers in the 1960s. The concept gained prominence with Donald Knuth's "The Art of Computer Programming," where circular data structures were explored as fundamental algorithmic building blocks.

In systems programming, circular iterators became essential for implementing ring buffers, round-robin schedulers, and audio processing pipelines. The zero-copy approach pioneered in languages like C found new expression in Rust's ownership system, enabling memory-safe circular iteration without runtime overhead.

CIter's unique approach of preserving index information alongside values reflects modern needs in data processing, where maintaining context about element positions is crucial for algorithms like circular convolution, periodic data analysis, and rotating window operations. The modulo arithmetic approach used in CIter reflects decades of optimization in circular addressing, commonly found in digital signal processing and embedded systems where efficient wraparound behavior is critical for real-time performance.

---

## About

This project is an open-source component of [js0.site ⋅ Refactoring the Internet Plan](https://js0.site).

We are redefining the development paradigm of the Internet in a componentized way. Welcome to follow us:

- [Google Group](https://groups.google.com/g/js0-site)
- [js0site.bsky.social](https://bsky.app/profile/js0site.bsky.social)

---

<a id="zh"></a>

# CIter: Rust 零拷贝循环迭代器

## 目录导航

- [项目概述](#项目概述)
- [核心特性](#核心特性)
- [使用示例](#使用示例)
- [API 参考](#api-参考)
- [设计理念](#设计理念)
- [技术栈](#技术栈)
- [项目结构](#项目结构)
- [历史背景](#历史背景)

## 项目概述

CIter 是 Rust 语言的轻量级零拷贝循环迭代器库，支持从任意起始位置高效遍历切片。迭代器在切片边界处自动环绕，提供无缝的循环数据访问，同时保留索引信息，无需内存分配或数据拷贝。

## 核心特性

- **零拷贝设计**: 直接引用原始切片数据，无内存分配
- **灵活起始位置**: 支持从切片内任意索引开始迭代
- **自动环绕**: 到达末尾后自动从头开始，实现无缝循环
- **位置跟踪**: 内置位置跟踪功能，监控迭代器当前状态
- **索引信息**: 返回每个元素的索引和值
- **随机起始支持**: 通过 `rand` 特性支持随机起始位置
- **内存安全**: 利用 Rust 所有权系统确保内存访问安全
- **泛型实现**: 适用于任意切片类型 `&[T]`

## 使用示例

### 基础循环迭代

```rust
use citer::CIter;

let data = [1, 2, 3, 4, 5];
let iter = CIter::new(&data, 2); // 从索引 2 开始
let result: Vec<_> = iter.collect();
// 结果: [(2, &3), (3, &4), (4, &5), (0, &1), (1, &2)]
```

### 位置跟踪

```rust
let data = [10, 20, 30];
let iter = CIter::new(&data, 2);
println!("当前位置: {}", iter.pos()); // 输出: 1
```

### 随机起始位置

```rust
// 需要启用 "rand" 特性
let data = [1, 2, 3, 4, 5];
let iter = CIter::rand(&data);
let result: Vec<_> = iter.collect();
// 结果: 从随机位置开始的随机排列，包含索引信息
```

### 手动迭代

```rust
let data = [1, 2, 3];
let mut iter = CIter::new(&data, 1);

assert_eq!(iter.next(), Some((1, &2)));
assert_eq!(iter.next(), Some((2, &3)));
assert_eq!(iter.next(), Some((0, &1)));
assert_eq!(iter.next(), None); // 迭代器耗尽
```

### 仅提取值

```rust
let data = [1, 2, 3, 4, 5];
let iter = CIter::new(&data, 2);
let values: Vec<&i32> = iter.map(|(_, &value)| value).collect();
// 结果: [3, 4, 5, 1, 2]
```

## API 参考

### `CIter<'a, T>`

主要循环迭代器结构体，包含生命周期参数 `'a` 和泛型类型 `T`。

#### 字段

- `idx: usize` - 当前索引位置
- `li: &'a [T]` - 切片数据的引用
- `ed: usize` - 已访问元素数量

#### 方法

- `new(li: &'a [T], pos: usize) -> Self`
  - 创建从指定位置开始的循环迭代器
  - 位置直接使用，边界检查在迭代时通过模运算处理

- `pos(&self) -> usize`
  - 返回当前逻辑位置（从 0 开始）
  - 非零索引返回 `idx - 1`，零索引返回 `0`

- `rand(li: &'a [T]) -> Self` (需要 `rand` 特性)
  - 创建随机起始位置的迭代器
  - 使用线程本地随机数生成器

#### 迭代器实现

实现标准 `Iterator` 特征:

- `type Item = (usize, &'a T)` - 返回 (索引, 值的引用) 元组
- `fn next(&mut self) -> Option<Self::Item>`

## 设计理念

库采用极简主义方法，专注于性能和安全性：

```mermaid
graph TD
  A[切片输入] --> B[CIter::new]
  B --> C[位置计算]
  C --> D[迭代器创建]
  D --> E[next 调用]
  E --> F{已访问元素 < 长度}
  F -->|是| G[通过模运算计算索引]
  G --> H[返回 (索引, 值) 元组]
  F -->|否| I[返回 None]
  H --> J[递增计数器]
  J --> E
  I --> K[迭代器耗尽]
```

### 核心设计原则

1. **零拷贝架构**: 引用原始数据而非复制
2. **有界迭代**: 保证访问所有元素一次后终止
3. **模运算**: 高效处理位置环绕
4. **索引保留**: 在整个迭代过程中保持原始索引信息
5. **生命周期安全**: 确保迭代器不会超出引用数据的生命周期

### 模块交互流程

- `CIter::new()` 初始化迭代器状态
- `Iterator::next()` 实现核心循环逻辑和模运算
- 位置跟踪维护当前状态，无额外内存分配
- 可选 `rand` 特性提供随机起始位置
- 索引信息在返回的元组中保留

## 技术栈

- **编程语言**: Rust 2024 版本
- **核心依赖**:
  - `aok` (0.1.18) - 结果处理工具
- **可选依赖**:
  - `rand` (0.9.2) - 随机起始位置支持
- **开发依赖**:
  - `aok` (0.1.18) - 测试结果处理
  - `log` (0.4.29) - 日志基础设施
  - `loginit` (0.1.18) - 日志初始化
  - `static_init` (1.0.4) - 静态初始化
  - `log` (0.1.43) - 结构化日志
  - `log_init` (0.1.34) - 本地日志初始化工具

## 项目结构

```
citer/
├── src/
│   └── lib.rs          # 核心 CIter 实现
├── tests/
│   └── main.rs         # 集成测试
├── readme/
│   ├── en.md          # 英文文档
│   └── zh.md          # 中文文档
├── Cargo.toml         # 项目配置
└── test.sh           # 测试执行脚本
```

### 关键组件

- **`CIter` 结构体**: 主要迭代器实现，包含位置和元素跟踪
- **Iterator 特征**: 标准 Rust 迭代器接口，返回 (索引, 值) 元组
- **特性门控**: 编译时标志控制的可选功能
- **全面测试**: 单元测试和集成测试覆盖

## 历史背景

循环迭代器在计算机科学中有着深厚的历史根源，可追溯到 1960 年代早期的循环缓冲区工作。这个概念在 Donald Knuth 的《计算机程序设计艺术》中得到了重视，其中循环数据结构被探索为基础算法构建块。

在系统编程中，循环迭代器成为实现环形缓冲区、轮询调度器和音频处理管道的关键技术。C 语言中开创的零拷贝方法在 Rust 的所有权系统中找到了新的表达方式，实现了内存安全的循环迭代，且无运行时开销。

CIter 在保留值的同时保留索引信息的独特方法反映了现代数据处理的需求，在循环卷积、周期性数据分析和旋转窗口操作等算法中，维护元素位置的上下文信息至关重要。CIter 中使用的模运算方法反映了循环寻址数十年的优化历程，这种方法在数字信号处理和嵌入式系统中很常见，在这些领域中，高效的环绕行为对实时性能至关重要。

---

## 关于

本项目为 [js0.site ⋅ 重构互联网计划](https://js0.site) 的开源组件。

我们正在以组件化的方式重新定义互联网的开发范式，欢迎关注：

- [谷歌邮件列表](https://groups.google.com/g/js0-site)
- [js0site.bsky.social](https://bsky.app/profile/js0site.bsky.social)
