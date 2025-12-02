[English](#en) | [中文](#zh)

---

<a id="en"></a>

# kvid : Global Unique ID Generator Based on Redis/Kvrocks

- [Introduction](#introduction)
- [Usage](#usage)
- [Design and Implementation](#design-and-implementation)
- [Tech Stack](#tech-stack)
- [Directory Structure](#directory-structure)
- [Competitors](#competitors)
- [Comparison of ID Generation Algorithms](#comparison-of-id-generation-algorithms)
- [History](#history)

## Introduction

`kvid` is a distributed unique ID generator based on Redis or Kvrocks. It guarantees globally unique IDs that are trend-increasing. It is designed to be robust, high-performance, and easy to integrate into Rust projects.

Key features include:
- **Global Uniqueness**: Ensures no duplicate IDs are generated across the distributed system.
- **Trend Increasing**: IDs are generated in an increasing order, which is beneficial for database indexing.
- **High Performance**: Utilizes batch fetching (step-based) to minimize network round-trips to Redis/Kvrocks.
- **Dynamic Step Adjustment**: Automatically adjusts the batch size based on consumption rate to balance performance and ID continuity.
- **Static Global Variable Support**: Can be directly declared as a `static` global variable in Rust, simplifying usage across the application.

## Usage

Add `kvid` to `Cargo.toml`.

### Basic Example

`kvid` allows declaring the generator as a static global variable, making it accessible throughout the application without passing instances around.

```rust
use std::time::Duration;
use aok::{OK, Void};
use kvid::KvId;
use log::info;

// Initialize logger (optional, depending on your setup)
#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
}

// Declare as a static global variable
pub static KVID_TEST: KvId = KvId::new("test");

#[tokio::test]
async fn test() -> Void {
  // Initialize global Redis/Kvrocks connection (Required for xkv to connect to Redis/Kvrocks)
  xboot::init().await?;

  for i in 0..300 {
    // Generate next ID
    let id = KVID_TEST.next().await?;
    info!("{}", id);

    if i > 5 {
      tokio::time::sleep(Duration::from_secs(1)).await;
    }
  }
  OK
}
```

## Design and Implementation

### Configuration & Constants

The following constants are defined in `src/lib.rs` and control the behavior of the generator:

-   **`FETCH_DURATION` (Default: 600s / 10 minutes)**:
    The target duration for a batch of IDs to last. The algorithm tries to adjust the `step` (batch size) so that a fetch request happens approximately every 10 minutes.
    -   If IDs are consumed **faster** than this duration, the step size **doubles** (up to `STEP_MAX`) to reduce network frequency.
    -   If IDs are consumed **slower** than this duration, the step size **halves** to prevent holding too many unused IDs during low traffic periods.

-   **`STEP_MAX` (Default: 1,000,000)**:
    The maximum number of IDs that can be fetched in a single request. This prevents the step size from growing indefinitely.

### Core Logic

The core logic resides in the `KvId` struct. It maintains a local range of IDs and fetches a new range (step) from the Redis/Kvrocks backend when the local range is exhausted.

1.  **Initialization**: `KvId` is initialized with a name (key).
2.  **ID Generation (`next`)**:
    - Checks if there are available IDs in the local buffer (`id < max`).
    - If yes, increments the local `id` and returns it.
    - If no, it triggers a fetch operation.
3.  **Fetching from Backend**:
    - Uses `HINCRBY` command on Redis/Kvrocks to atomically increment the maximum ID for the given key by `step`.
    - Updates local `max` and `id` based on the response.
4.  **Dynamic Step Adjustment**:
    - The system calculates the time elapsed (`cost`) since the last fetch.
    - **Increase Step**: If `cost <= FETCH_DURATION` (high load), `step = step * 2`.
    - **Decrease Step**: If `cost > FETCH_DURATION` (low load), `step = max(1, step / 2)`.
    - This self-tuning mechanism ensures high performance under load while minimizing waste during idle times.

### Data Structures (`lib.rs`)

-   **`KvId`**: The main struct exposed to the user.
    -   `name`: The key name used in Redis.
    -   `inner`: A `Mutex` protected `Inner` state.
-   **`Inner`**: Internal state of the generator.
    -   `id`: Current ID available for distribution.
    -   `max`: The maximum ID in the current allocated range.
    -   `step`: Current batch size to fetch from backend.
    -   `ts`: Timestamp of the last fetch.

## Tech Stack

-   **Rust**: Core language.
-   **Redis / Kvrocks**: Backend storage for atomic counters.
-   **fred**: Async Redis client.
-   **parking_lot**: Efficient Mutex implementation.
-   **tokio**: Async runtime.

## Directory Structure

```
.
├── Cargo.toml      # Project configuration
├── README.mdt      # README template
├── readme/         # Documentation files
│   ├── en.md       # English README
│   └── zh.md       # Chinese README
├── src/            # Source code
│   └── lib.rs      # Library entry point
└── tests/          # Integration tests
    └── main.rs     # Usage demonstration
```

## Competitors

Distributed ID generation is a common requirement. Here are some similar projects:

-   **Baidu Uidgenerator**: Java-based, Snowflake algorithm variant. High performance but relies on Snowflake's time-dependency.
-   **Meituan Leaf**: Supports both Segment mode (database) and Snowflake mode (ZooKeeper). Segment mode is similar to `kvid`'s approach.
-   **Didi TinyID**: Java-based, Segment mode only. Focuses on high availability and multi-db support.

`kvid` distinguishes itself by being written in Rust, offering high performance with low footprint, and specifically optimizing for ease of use with static global variables.

## Comparison of ID Generation Algorithms

To better understand `kvid`'s position, here is a comparison of common distributed ID generation algorithms:

### 1. UUID (Universally Unique Identifier)
-   **Principle**: 128-bit identifier generated based on timestamp, random numbers, or MAC address.
-   **Pros**:
    -   Simple to implement, no network interaction needed.
    -   Globally unique without coordination.
-   **Cons**:
    -   **Too long**: 128 bits (32 hex chars) is inefficient for storage and indexing.
    -   **Not sortable**: Randomness (v4) causes page splitting in B+Tree indexes, hurting database performance.
    -   **Information leakage**: v1 contains MAC address.

### 2. Database Auto-increment
-   **Principle**: Rely on database's `AUTO_INCREMENT` feature.
-   **Pros**:
    -   Simple, strictly increasing.
-   **Cons**:
    -   **Single point of failure**: Database becomes the bottleneck.
    -   **Hard to scale**: Difficult to merge data from multiple databases later.

### 3. Snowflake (Twitter)
-   **Principle**: 64-bit integer: 1 bit sign + 41 bits timestamp + 10 bits machine ID + 12 bits sequence.
-   **Pros**:
    -   High performance (millions of IDs/sec).
    -   Time-ordered (roughly).
    -   No network overhead (generated locally).
-   **Cons**:
    -   **Clock dependency**: Strongly relies on system clock. Clock rollback can cause duplicate IDs or service unavailability.
    -   **Machine ID management**: Requires a mechanism (like ZooKeeper) to assign unique machine IDs.

### 4. Segment Mode (kvid / Meituan Leaf)
-   **Principle**: Pre-allocate a range (step) of IDs from a central store (Redis/DB) and issue them from memory.
-   **Pros**:
    -   **High Performance**: Database is accessed only once per step (e.g., every 1000 IDs).
    -   **No Clock Dependency**: Immune to clock rollback issues.
    -   **Trend Increasing**: Friendly to database indexing.
-   **Cons**:
    -   **ID Gaps**: If the service restarts, unused IDs in the current step are lost (but uniqueness is preserved).
    -   **Central Dependency**: Relies on the availability of the central store (Redis/Kvrocks), though load is very low.

## History

The need for distributed unique IDs arose with the explosion of web-scale applications. Traditional database auto-increment keys became a bottleneck in sharded databases. Twitter's **Snowflake** (2010) was a pioneer, using time-based bit manipulation to generate IDs without coordination. However, Snowflake depends heavily on system clocks. Database-based "Segment" approaches (like Flickr's ticket server and later Meituan Leaf's segment mode) emerged to solve clock dependency issues by allocating blocks of IDs. `kvid` follows the Segment pattern, leveraging modern Redis/Kvrocks for speed and atomicity, combined with Rust's safety and concurrency features.

---

## About

This project is an open-source component of [js0.site ⋅ Refactoring the Internet Plan](https://js0.site).

We are redefining the development paradigm of the Internet in a componentized way. Welcome to follow us:

* [Google Group](https://groups.google.com/g/js0-site)
* [js0site.bsky.social](https://bsky.app/profile/js0site.bsky.social)

---

<a id="zh"></a>

# kvid : 基于 Redis/Kvrocks 的全局唯一 ID 生成器

- [项目介绍](#项目介绍)
- [使用演示](#使用演示)
- [设计思路](#设计思路)
- [技术堆栈](#技术堆栈)
- [目录结构](#目录结构)
- [竞品对比](#竞品对比)
- [主流 ID 生成算法对比](#主流-id-生成算法对比)
- [历史背景](#历史背景)

## 项目介绍

`kvid` 是基于 Redis 或 Kvrocks 的分布式全局唯一 ID 生成器。保证生成的 ID 全局唯一且整体呈递增趋势。设计目标是稳健、高性能，并易于集成到 Rust 项目中。

主要特性：
- **全局唯一**：确保在分布式系统中不会生成重复 ID。
- **趋势递增**：ID 按序生成，有利于数据库索引性能。
- **高性能**：采用号段模式（Batch Fetching），极大减少对 Redis/Kvrocks 的网络请求频率。
- **动态步长**：根据 ID 消费速率自动调整步长，平衡性能与 ID 连续性。
- **支持静态全局变量**：Rust 中可直接声明为 `static` 全局变量，无需在函数间传递实例，使用极其便捷。

## 使用演示

在 `Cargo.toml` 中添加 `kvid`。

### 基础示例

`kvid` 支持声明为静态全局变量，可在应用任意位置直接调用。

```rust
use std::time::Duration;
use aok::{OK, Void};
use kvid::KvId;
use log::info;

// 初始化日志（可选）
#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
}

// 声明静态全局变量
pub static KVID_TEST: KvId = KvId::new("test");

#[tokio::test]
async fn test() -> Void {
  // 初始化全局 Redis/Kvrocks 连接(使用xkv连接redis/kvrocks数据库必须先这样初始化)
  xboot::init().await?;

  for i in 0..300 {
    // 生成下一个 ID
    let id = KVID_TEST.next().await?;
    info!("{}", id);

    if i > 5 {
      tokio::time::sleep(Duration::from_secs(1)).await;
    }
  }
  OK
}
```

## 设计思路

### 配置与常量

以下常量定义在 `src/lib.rs` 中，用于控制生成器的行为：

-   **`FETCH_DURATION` (默认: 600秒 / 10分钟)**：
    一个号段（Batch）预期的持续时间。算法会尝试调整 `step`（步长），使得大约每 10 分钟进行一次网络请求。
    -   如果 ID 消费速度**快于**此时间，步长**翻倍**（上限为 `STEP_MAX`），以减少网络交互频率。
    -   如果 ID 消费速度**慢于**此时间，步长**减半**，以防止在低流量期间持有过多未使用的 ID。

-   **`STEP_MAX` (默认: 1,000,000)**：
    单次请求获取的最大 ID 数量。防止步长无限增长。

### 核心逻辑

核心逻辑封装于 `KvId` 结构体。其内部维护一段本地 ID 缓存，当缓存耗尽时，从 Redis/Kvrocks 后端获取新的号段（Step）。

1.  **初始化**：使用名称（Key）初始化 `KvId`。
2.  **ID 生成 (`next`)**：
    - 检查本地缓存是否有可用 ID (`id < max`)。
    - 若有，递增本地 `id` 并返回。
    - 若无，触发远程获取操作。
3.  **后端获取**：
    - 使用 Redis/Kvrocks 的 `HINCRBY` 命令，原子性地增加对应 Key 的最大值。
    - 根据返回值更新本地 `max` 和 `id`。
4.  **动态步长调整**：
    - 系统计算距离上次获取的时间间隔 (`cost`)。
    - **增加步长**：若 `cost <= FETCH_DURATION`（高负载），则 `step = step * 2`。
    - **减小步长**：若 `cost > FETCH_DURATION`（低负载），则 `step = max(1, step / 2)`。
    - 这种自适应机制确保了在高负载下的高性能，同时在空闲时减少 ID 浪费。

### 数据结构 (`lib.rs`)

-   **`KvId`**：对外暴露的主结构体。
    -   `name`: Redis 中使用的 Key 名称。
    -   `inner`: `Mutex` 保护的内部状态 `Inner`。
-   **`Inner`**：生成器内部状态。
    -   `id`: 当前可分配的 ID。
    -   `max`: 当前号段的最大 ID。
    -   `step`: 当前从后端获取的步长。
    -   `ts`: 上次获取的时间戳。

## 技术堆栈

-   **Rust**: 核心开发语言。
-   **Redis / Kvrocks**: 用于原子计数器的后端存储。
-   **fred**: 异步 Redis 客户端。
-   **parking_lot**: 高效 Mutex 实现。
-   **tokio**: 异步运行时。

## 目录结构

```
.
├── Cargo.toml      # 项目配置
├── README.mdt      # README 模板
├── readme/         # 文档目录
│   ├── en.md       # 英文文档
│   └── zh.md       # 中文文档
├── src/            # 源代码
│   └── lib.rs      # 库入口
└── tests/          # 集成测试
    └── main.rs     # 使用演示
```

## 竞品对比

分布式 ID 生成是常见需求，类似项目包括：

-   **百度 Uidgenerator**：基于 Java，Snowflake 算法变种。高性能，但依赖机器时钟。
-   **美团 Leaf**：支持号段模式（数据库）和 Snowflake 模式（ZooKeeper）。其号段模式与 `kvid` 思路类似。
-   **滴滴 TinyID**：基于 Java，仅支持号段模式。侧重高可用和多 DB 支持。

`kvid` 的优势在于使用 Rust 编写，资源占用低，性能卓越，且专门针对 Rust 开发习惯优化，支持静态全局变量调用。

## 主流 ID 生成算法对比

为了更好地理解 `kvid` 的定位，以下是常见分布式 ID 生成算法的对比：

### 1. UUID (Universally Unique Identifier)
-   **原理**：基于时间戳、随机数或 MAC 地址生成的 128 位标识符。
-   **优点**：
    -   实现简单，无需网络交互。
    -   全球唯一，无需协调。
-   **缺点**：
    -   **太长**：128 位（32 个十六进制字符），存储和索引效率低。
    -   **无序**：随机性（v4）导致 B+Tree 索引频繁分裂，严重影响数据库性能。
    -   **信息泄露**：v1 版本包含 MAC 地址。

### 2. 数据库自增 ID
-   **原理**：依赖数据库的 `AUTO_INCREMENT` 特性。
-   **优点**：
    -   简单，严格递增。
-   **缺点**：
    -   **单点故障**：数据库成为性能瓶颈。
    -   **难以扩展**：分库分表后难以合并数据。

### 3. Snowflake (Twitter 雪花算法)
-   **原理**：64 位整数：1 位符号 + 41 位时间戳 + 10 位机器 ID + 12 位序列号。
-   **优点**：
    -   高性能（百万级 QPS）。
    -   趋势递增（基于时间）。
    -   无网络开销（本地生成）。
-   **缺点**：
    -   **强依赖时钟**：系统时钟回拨会导致 ID 重复或服务不可用。
    -   **机器 ID 管理**：需要额外的机制（如 ZooKeeper）来分配唯一的机器 ID。

### 4. 号段模式 (kvid / 美团 Leaf)
-   **原理**：从中心存储（Redis/DB）预申请一段 ID（步长），然后在内存中分配。
-   **优点**：
    -   **高性能**：每个步长（如 1000 个 ID）仅访问一次数据库。
    -   **无时钟依赖**：完全免疫时钟回拨问题。
    -   **趋势递增**：对数据库索引友好。
-   **缺点**：
    -   **ID 空洞**：服务重启时，当前步长内未使用的 ID 会丢失（但保证唯一性）。
    -   **依赖中心**：依赖中心存储（Redis/Kvrocks）的高可用，但负载极低。

## 历史背景

随着互联网应用规模爆发，分布式唯一 ID 需求应运而生。传统数据库自增主键在分库分表场景下成为瓶颈。Twitter 的 **Snowflake** (2010) 开创性地利用时间戳位运算生成 ID，无需中心化协调，但强依赖系统时钟。基于数据库的“号段模式”（如 Flickr 的 Ticket Server 及后来的美团 Leaf）通过预分配 ID 块解决了时钟依赖问题。`kvid` 沿用号段模式，利用现代 Redis/Kvrocks 的原子性与速度，结合 Rust 的安全并发特性，提供更优解决方案。

---

## 关于

本项目为 [js0.site ⋅ 重构互联网计划](https://js0.site) 的开源组件。

我们正在以组件化的方式重新定义互联网的开发范式，欢迎关注：

* [谷歌邮件列表](https://groups.google.com/g/js0-site)
* [js0site.bsky.social](https://bsky.app/profile/js0site.bsky.social)
