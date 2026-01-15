[English](#en) | [中文](#zh)

---

<a id="en"></a>

# ts_ : High-performance, lightweight time measurement for Rust

`ts_` is a simplified wrapper around the `coarsetime` crate, designed to provide extremely fast and easy-to-use time measurement utilities. It focuses on speed and API stability, making it ideal for performance-critical applications where high-precision (but not necessarily atomic-clock level) timestamps are needed without the overhead of standard library calls.

## Table of Contents

- [Features](#features)
- [Usage](#usage)
- [Design Philosophy](#design-philosophy)
- [Tech Stack](#tech-stack)
- [Directory Structure](#directory-structure)
- [API Reference](#api-reference)
- [History: The Quest for Speed](#history-the-quest-for-speed)

## Features

*   **High Performance**: Leverages `CLOCK_MONOTONIC_COARSE` on Linux systems for minimal overhead.
*   **Simplicity**: Exposes a straightforward API with `nano()`, `milli()`, and `sec()` functions.
*   **Stability**: Consistent behavior across different platforms, avoiding runtime panics common in some standard library implementations.
*   **Lightweight**: Minimal dependencies and optimized for speed.
*   **Flexible Precision**: Choose between nanoseconds, milliseconds, or seconds based on your needs.

## Usage

Add `ts_` to your `Cargo.toml`:

```toml
[dependencies]
ts_ = "0.1.2"
```

Enable the features you need:

```toml
[dependencies]
ts_ = { version = "0.1.2", features = ["nano", "milli", "sec"] }
```

### Example

Here is a simple example demonstrating how to retrieve the current time in different precisions:

```rust
use aok::{OK, Void};
use log::info;

#[test]
fn test() -> Void {
  // Get current time in nanoseconds (requires "nano" feature)
  info!("{}", ts_::nano());

  // Get current time in milliseconds (requires "milli" feature)
  info!("{}", ts_::milli());

  // Get current time in seconds (requires "sec" feature)
  info!("{}", ts_::sec());

  OK
}
```

## Design Philosophy

The core design philosophy of `ts_` is **"Speed over Accuracy"**.

1.  **Underlying Mechanism**: It wraps `coarsetime`, which uses `CLOCK_MONOTONIC_COARSE` on Linux. This avoids expensive system calls by reading the time directly from a memory page updated by the kernel (vDSO).
2.  **Simplified API**: Instead of dealing with complex `Duration` or `Instant` objects for simple timestamp needs, `ts_` provides direct access to `u64` values representing time.
3.  **Feature Flags**: Users can opt-in for `nano` or `sec` precision features to keep the compilation minimal.

## Tech Stack

*   **Language**: Rust
*   **Core Dependency**: `coarsetime`
*   **Repository**: https://github.com/js0-site/rust.git
*   **License**: MulanPSL-2.0

## Directory Structure

```
.
├── src/
│   └── lib.rs      # Core implementation exporting nano() and sec()
├── tests/
│   └── main.rs     # Integration tests and usage examples
├── Cargo.toml      # Project configuration
└── readme/         # Documentation
```

## API Reference

### `nano()`

```rust
pub fn nano() -> u64
```

Returns the current time in nanoseconds since the epoch. Requires the `nano` feature.

### `milli()`

```rust
pub fn milli() -> u64
```

Returns the current time in milliseconds since the epoch. Requires the `milli` feature.

### `sec()`

```rust
pub fn sec() -> u64
```

Returns the current time in seconds since the epoch. Requires the `sec` feature.

## History: The Quest for Speed

In the early days of Linux, getting the time was always a system call. A system call involves switching from user mode to kernel mode, which is a relatively expensive operation for the CPU. For high-performance applications that needed to check the time thousands or millions of times per second (like high-frequency trading or high-throughput servers), this overhead was significant.

To solve this, the Linux kernel introduced the **vDSO (Virtual Dynamic Shared Object)** mechanism. This allows the kernel to map a small area of memory into the user space of every process. This memory contains frequently used data, such as the current time.

`CLOCK_MONOTONIC_COARSE` was born from this innovation. It reads the time directly from this shared memory without triggering a context switch. While it might be slightly less precise than a full system call (updating only on timer ticks), it is orders of magnitude faster. `ts_` (via `coarsetime`) harnesses this power to give your Rust applications blazing fast time measurements.

---

## About

This project is an open-source component of [js0.site ⋅ Refactoring the Internet Plan](https://js0.site).

We are redefining the development paradigm of the Internet in a componentized way. Welcome to follow us:

* [Google Group](https://groups.google.com/g/js0-site)
* [js0site.bsky.social](https://bsky.app/profile/js0site.bsky.social)

---

<a id="zh"></a>

# ts_ : Rust 高性能轻量级时间测量库

`ts_` 是 `coarsetime` crate 的简化封装，旨在提供极速且易用的时间测量工具。它专注于速度和 API 稳定性，非常适合那些需要高性能时间戳但不需要原子钟级精度的应用场景，避免了标准库调用的额外开销。

## 目录

- [功能特性](#功能特性)
- [使用演示](#使用演示)
- [设计思路](#设计思路)
- [技术堆栈](#技术堆栈)
- [目录结构](#目录结构)
- [API 参考](#api-参考)
- [历史趣闻](#历史趣闻)

## 功能特性

*   **高性能**：在 Linux 系统上利用 `CLOCK_MONOTONIC_COARSE`，将开销降至最低。
*   **极简易用**：暴露 `nano()`、`milli()` 和 `sec()` 三个核心函数，上手即用。
*   **跨平台稳定**：在不同平台上保持一致的行为，避免了标准库在某些平台上可能出现的运行时 panic。
*   **轻量级**：依赖极少，专为速度优化。
*   **灵活精度**：根据需求选择纳秒、毫秒或秒级精度。

## 使用演示

在 `Cargo.toml` 中添加 `ts_`：

```toml
[dependencies]
ts_ = "0.1.2"
```

按需启用特性：

```toml
[dependencies]
ts_ = { version = "0.1.2", features = ["nano", "milli", "sec"] }
```

### 示例代码

以下代码展示了如何获取不同精度的时间戳：

```rust
use aok::{OK, Void};
use log::info;

#[test]
fn test() -> Void {
  // 获取当前纳秒时间戳（需要 "nano" 特性）
  info!("{}", ts_::nano());

  // 获取当前毫秒时间戳（需要 "milli" 特性）
  info!("{}", ts_::milli());

  // 获取当前秒级时间戳（需要 "sec" 特性）
  info!("{}", ts_::sec());

  OK
}
```

## 设计思路

`ts_` 的核心设计理念是 **“速度优先”**。

1.  **底层机制**：封装了 `coarsetime`，在 Linux 上使用 `CLOCK_MONOTONIC_COARSE`。通过读取内核更新的内存页（vDSO）直接获取时间，避免了昂贵的系统调用。
2.  **简化 API**：摒弃了复杂的 `Duration` 或 `Instant` 对象，直接提供 `u64` 类型的时间数值，满足最纯粹的时间获取需求。
3.  **按需加载**：通过 `nano` 和 `sec` 特性标志（Feature Flags），用户可以按需编译所需功能。

## 技术堆栈

*   **编程语言**: Rust
*   **核心依赖**: `coarsetime`
*   **代码仓库**: https://github.com/js0-site/rust.git
*   **许可证**: MulanPSL-2.0

## 目录结构

```
.
├── src/
│   └── lib.rs      # 核心实现，导出 nano() 和 sec()
├── tests/
│   └── main.rs     # 集成测试与使用示例
├── Cargo.toml      # 项目配置
└── readme/         # 文档目录
```

## API 参考

### `nano()`

```rust
pub fn nano() -> u64
```

返回自纪元以来的当前时间（纳秒）。需要启用 `nano` 特性。

### `milli()`

```rust
pub fn milli() -> u64
```

返回自纪元以来的当前时间（毫秒）。需要启用 `milli` 特性。

### `sec()`

```rust
pub fn sec() -> u64
```

返回自纪元以来的当前时间（秒）。需要启用 `sec` 特性。

## 历史趣闻：对速度的极致追求

在 Linux 的早期，获取时间总是一次系统调用（System Call）。系统调用意味着 CPU 需要从用户态切换到内核态，这对于处理器来说是一个相对昂贵的操作。对于那些需要每秒成千上万次检查时间的高性能应用（如高频交易或高吞吐量服务器）来说，这种开销是不可忽视的。

为了解决这个问题，Linux 内核引入了 **vDSO (Virtual Dynamic Shared Object)** 机制。它允许内核将一小块内存映射到每个进程的用户空间。这块内存包含了经常使用的数据，比如当前时间。

`CLOCK_MONOTONIC_COARSE` 正是这一创新的产物。它直接从这块共享内存中读取时间，而无需触发上下文切换。虽然它的精度可能略低于完整的系统调用（仅在定时器滴答时更新），但它的速度却快了几个数量级。`ts_`（通过 `coarsetime`）正是利用了这一机制，为您的 Rust 应用提供闪电般的时间测量能力。

---

## 关于

本项目为 [js0.site ⋅ 重构互联网计划](https://js0.site) 的开源组件。

我们正在以组件化的方式重新定义互联网的开发范式，欢迎关注：

* [谷歌邮件列表](https://groups.google.com/g/js0-site)
* [js0site.bsky.social](https://bsky.app/profile/js0site.bsky.social)
