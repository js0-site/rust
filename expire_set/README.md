[English](#en) | [中文](#zh)

---

<a id="en"></a>

# expire_set : High-performance concurrent expiration set

A high-performance, concurrent set with automatic item expiration, implemented using `unsafe` raw pointers for maximum efficiency.

## Table of Contents

- [Introduction](#introduction)
- [Features](#features)
- [Usage](#usage)
- [Design Philosophy](#design-philosophy)
- [API Documentation](#api-documentation)
- [Technology Stack](#technology-stack)
- [Directory Structure](#directory-structure)
- [Historical Trivia](#historical-trivia)

## Introduction

`expire_set` is a specialized Rust library designed for high-throughput scenarios where items need to expire after a short duration, such as **caching 404 request paths** to prevent DoS attacks.

Unlike traditional TTL caches that store a timestamp for *every single item*, `expire_set` uses a double-buffering strategy. This approach eliminates the memory overhead of per-item timestamps and the CPU overhead of checking them, making it extremely memory-efficient and fast.

## Features

- **Memory Efficient**: Does **not** store expiration timestamps for individual items. Saves significant memory when caching millions of small items (like IP addresses or URLs).
- **Ideal for Short-Lived Cache**: Perfect for use cases like "expire after 1 minute," such as 404 flooding protection or deduplication buffers.
- **High Performance**: Uses `unsafe` raw pointers and `AtomicUsize` to avoid `Arc` reference counting overhead.
- **Concurrency**: Built on `DashSet` for thread-safe, concurrent access.
- **Automatic Bulk Expiration**: Background timer rotates buffers to expire old items in bulk, rather than scanning for expired items one by one.
- **Zero Overhead Sharing**: State is shared between the timer and the main struct using raw pointers.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
expire_set = "0.1.0"
```

Example usage:

```rust
use expire_set::ExpireSet;
use std::time::Duration;

#[tokio::main]
async fn main() {
    // Create a set where items expire every 10 seconds
    let set = ExpireSet::<String>::new(10);

    // Insert items
    set.insert("key1".to_string());
    
    // Check existence
    if set.contains(&"key1".to_string()) {
        println!("Key exists!");
    }

    // Wait for expiration
    tokio::time::sleep(Duration::from_secs(25)).await;
    
    // Item should be gone
    assert!(!set.contains(&"key1".to_string()));
}
```

## Design Philosophy

The core design is based on a **Double Buffering** (or Rotating Cache) mechanism:

1.  **Two Buffers**: The struct holds two `DashSet` instances.
2.  **Atomic Index**: An `AtomicUsize` indicates the "current" active buffer (0 or 1).
3.  **Insertion**: New items are always inserted into the `current` buffer.
4.  **Querying**: `contains` checks *both* buffers to ensure items are valid until they are fully cleared.
5.  **Rotation**: A background Tokio task sleeps for the `expire` duration. Upon waking, it flips the index (0 -> 1 or 1 -> 0) and clears the *new* current buffer (which holds the oldest data).

This approach avoids checking timestamps for every item. Instead, items expire in bulk when their buffer is cleared.

### Unsafe Optimization

To satisfy strict performance requirements, `Arc` is bypassed in favor of `unsafe` raw pointers (`*const`) and `Box::leak`.
-   Data is leaked to the heap with `'static` lifetime.
-   Pointers are wrapped in a `SendPtr` struct to allow passing them to the background task.
-   Memory is manually reclaimed in `Drop`.

## API Documentation

### `ExpireSet<K>`

The main struct. `K` must implement `Hash + Eq + Clone + Send + Sync + 'static`.

#### `fn new(expire: u64) -> Self`
Creates a new `ExpireSet`.
-   `expire`: The duration in seconds before the buffer rotates. Items live for roughly `expire` to `2 * expire` seconds.

#### `fn insert(&self, key: K)`
Inserts a key into the current active set.

#### `fn contains(&self, key: &K) -> bool`
Checks if the key exists in either the current or the previous set.

## Technology Stack

-   **Rust**: Core language.
-   **Tokio**: Async runtime for the background timer task.
-   **DashMap**: Concurrent associative array for storage.
-   **Atomic**: Standard library atomics for synchronization.

## Directory Structure

```
.
├── Cargo.toml          # Project configuration
├── readme/             # Documentation
│   ├── en.md           # English README
│   └── zh.md           # Chinese README
├── src/
│   └── lib.rs          # Source code (ExpireSet implementation)
└── tests/
    └── main.rs         # Integration tests
```

## Historical Trivia

**The Origin of Double Buffering**

The "rotating cache" technique used in this project is analogous to **Double Buffering** in computer graphics.

Double buffering originated in the late 1960s and became standard in the 1980s with systems like the **Amiga**. In graphics, it involves drawing to a hidden "back buffer" while displaying the "front buffer," then swapping them instantly to prevent screen tearing.

Similarly, `expire_set` writes to a "current" buffer while keeping the "previous" buffer available for reads. When the timer fires, it "swaps" the buffers (by changing the index) and clears the old one, ensuring a smooth transition and efficient bulk expiration, much like the artifact-free rendering in early graphics hardware.

---

## About

This project is an open-source component of [js0.site ⋅ Refactoring the Internet Plan](https://js0.site).

We are redefining the development paradigm of the Internet in a componentized way. Welcome to follow us:

* [Google Group](https://groups.google.com/g/js0-site)
* [js0site.bsky.social](https://bsky.app/profile/js0site.bsky.social)

---

<a id="zh"></a>

# expire_set : 基于非安全原始指针的高性能并发过期集合

使用 `unsafe` 原始指针实现的高性能并发集合，支持自动过期。

## 目录

- [简介](#简介)
- [特性](#特性)
- [使用演示](#使用演示)
- [设计思路](#设计思路)
- [API 文档](#api-文档)
- [技术堆栈](#技术堆栈)
- [目录结构](#目录结构)
- [历史小故事](#历史小故事)

## 简介

`expire_set` 是一个专为高吞吐量场景设计的 Rust 库，适用于需要短时间过期的项目，例如**缓存 404 请求路径**以防止 DoS 攻击。

与为*每个项目*存储时间戳的传统 TTL 缓存不同，`expire_set` 采用双缓冲策略。这种方法完全消除了存储每项时间戳的内存开销和检查时间戳的 CPU 开销，使其具有极高的内存效率和速度。

## 特性

-   **极致省内存**：**不**为单个项目保存过期时间戳。在缓存数百万个小对象（如 IP 地址或 URL）时，可节省大量内存。
-   **短时缓存利器**：非常适合“一分钟后过期”这类场景，如 404 洪水攻击防护或去重缓冲区。
-   **高性能**：使用 `unsafe` 原始指针和 `AtomicUsize`，避免 `Arc` 引用计数开销。
-   **高并发**：基于 `DashSet` 实现线程安全并发访问。
-   **批量自动过期**：后台定时器轮转缓冲区，批量过期旧项目，而非逐个扫描过期项。
-   **零开销共享**：通过原始指针在定时器任务和主结构体间共享状态。

## 使用演示

在 `Cargo.toml` 中添加：

```toml
[dependencies]
expire_set = "0.1.0"
```

代码示例：

```rust
use expire_set::ExpireSet;
use std::time::Duration;

#[tokio::main]
async fn main() {
    // 创建集合，每 10 秒轮转一次
    let set = ExpireSet::<String>::new(10);

    // 插入数据
    set.insert("key1".to_string());
    
    // 查询存在性
    if set.contains(&"key1".to_string()) {
        println!("Key exists!");
    }

    // 等待过期
    tokio::time::sleep(Duration::from_secs(25)).await;
    
    // 数据已清除
    assert!(!set.contains(&"key1".to_string()));
}
```

## 设计思路

核心设计基于 **双缓冲**（Double Buffering）或轮转缓存机制：

1.  **双缓冲区**：结构体持有两个 `DashSet` 实例。
2.  **原子索引**：使用 `AtomicUsize` 指示“当前”活跃缓冲区（0 或 1）。
3.  **写入**：新项目总是插入到 `current` 缓冲区。
4.  **查询**：`contains` 同时检查两个缓冲区，确保数据在完全清除前可用。
5.  **轮转**：后台 Tokio 任务休眠 `expire` 时长。唤醒后，切换索引（0 -> 1 或 1 -> 0）并清空*新*的当前缓冲区（其中包含最旧数据）。

此方法避免了为每个项目检查时间戳。项目在缓冲区清空时批量过期。

### Unsafe 优化

为满足严格性能要求，放弃 `Arc`，改用 `unsafe` 原始指针（`*const`）和 `Box::leak`。
-   数据泄漏到堆上，具有 `'static` 生命周期。
-   指针封装在 `SendPtr` 结构中，以便传递给后台任务。
-   在 `Drop` 中手动回收内存。

## API 文档

### `ExpireSet<K>`

主结构体。`K` 必须实现 `Hash + Eq + Clone + Send + Sync + 'static`。

#### `fn new(expire: u64) -> Self`
创建新 `ExpireSet`。
-   `expire`: 缓冲区轮转间隔（秒）。项目存活时间约为 `expire` 到 `2 * expire` 秒。

#### `fn insert(&self, key: K)`
将键插入当前活跃集合。

#### `fn contains(&self, key: &K) -> bool`
检查键是否存在于当前或上一个集合中。

## 技术堆栈

-   **Rust**: 核心语言。
-   **Tokio**: 用于后台定时器任务的异步运行时。
-   **DashMap**: 用于存储的并发关联数组。
-   **Atomic**: 标准库原子操作，用于同步。

## 目录结构

```
.
├── Cargo.toml          # 项目配置
├── readme/             # 文档目录
│   ├── en.md           # 英文说明
│   └── zh.md           # 中文说明
├── src/
│   └── lib.rs          # 源代码 (ExpireSet 实现)
└── tests/
    └── main.rs         # 集成测试
```

## 历史小故事

**双缓冲技术的起源**

本项目使用的“轮转缓存”技术类比于计算机图形学中的 **双缓冲**（Double Buffering）。

双缓冲起源于 20 世纪 60 年代末，并在 80 年代随着 **Amiga** 等系统的出现成为标准。在图形学中，它涉及在显示“前缓冲区”的同时向隐藏的“后缓冲区”绘图，然后瞬间交换两者以防止画面撕裂。

类似地，`expire_set` 写入“当前”缓冲区，同时保留“上一个”缓冲区供读取。当定时器触发时，通过更改索引“交换”缓冲区并清空旧缓冲区。这种机制确保了平滑过渡和高效批量过期，正如早期图形硬件实现无伪影渲染一样。

---

## 关于

本项目为 [js0.site ⋅ 重构互联网计划](https://js0.site) 的开源组件。

我们正在以组件化的方式重新定义互联网的开发范式，欢迎关注：

* [谷歌邮件列表](https://groups.google.com/g/js0-site)
* [js0site.bsky.social](https://bsky.app/profile/js0site.bsky.social)
