[English](n) | [中文](#zh)

---

<a id="en"></a>

# msgq : Robust Redis Stream Message Queue

Robust Redis Stream based message queue with auto-claim and retry handling.

## Table of Contents

- [Features](#features)
- [Usage](#usage)
- [Design](#design)
- [Tech Stack](#tech-stack)
- [Directory Structure](#directory-structure)
- [API Reference](#api-reference)
- [History](#history)

## Features

- **Redis Stream Based**: Utilizes `XREADGROUP` for efficient, scalable message consumption.
- **Consumer Groups**: Supports multiple consumers within a group for parallel processing.
- **Automatic Group Creation**: Automatically creates the Redis Stream group and consumer if they don't exist.
- **Reliable Delivery**: Implements auto-claiming of idle pending messages to prevent message loss.
- **Concurrent Processing**: Uses `tokio::spawn` with `async-scoped` to process messages concurrently.
- **Zero-Copy**: Leverages `async-scoped` to process borrowed data from the stream response without cloning, maximizing performance.
- **Message Chaining**: The `Parse::run` method can return `Some(Kv)` to add a new message to the queue, enabling workflow chaining.
- **Configurable Retries**: Automatic retry mechanism for failed messages, with a configurable limit.
- **Centralized Configuration**: A simple `Conf` struct to manage all connection and behavior settings.
- **Trait-Based Callbacks**: Uses a `Parse` trait for clear and reusable message processing and error handling logic.

## Usage

Define a struct and implement the `Parse` trait to handle your message processing and error logic.

```rust
use msgq::{Conf, Kv, Parse, ReadGroup};
use std::future::Future;
use aok::{OK, Void};
use log::info;

// 1. Define your message processor

struct MyParser;

impl Parse for MyParser {
  // 2. Implement the message processing logic
  async fn run(&self, kv: &Kv, retry: u64) -> aok::Result<Option<Kv>> {
    info!("run: {:?}, retry: {}", kv, retry);
    // Return Ok(None) for successful processing
    // Return Ok(Some(new_kv)) to add a new message to the queue
    // Return Err(...) to retry the message
    Ok(None)
  }

  // 3. Implement the error handling logic for messages that fail all retries
  async fn on_error(&self, kv: Kv, error: String) -> Void {
    info!("on_error: {:?}, error: {}", kv, error);
    OK
  }
}

#[tokio::main]
async fn main() -> Void {
  // 4. Initialize the environment (e.g. using xboot to set up global Redis client)
  // xboot::init().await?; 

  // 5. Configure the consumer
  let conf = Conf::new(
    "s1", // stream key
    "g1", // group name
    "c1", // consumer name
    5,    // block_sec: wait up to 5s for new messages
    60,   // claim_idle_sec: claim messages idle for 60s
    10,   // count: batch size
    3,    // max_retry: retry 3 times before on_error
  );

  // 6. Create a ReadGroup and run it
  ReadGroup::new(MyParser, conf).run().await?;
  OK
}
```

## Design

The `ReadGroup::run` method executes a continuous loop that ensures robust message processing:

1.  **Claim Idle Messages**: It first calls `XPENDING` to find messages that have been idle for longer than `claim_idle_ms` and claims them using `XAUTOCLAIM`. This ensures that messages from crashed or slow consumers are re-processed.
2.  **Fetch New Messages**: It then executes `XREADGROUP` with a `BLOCK` timeout to efficiently wait for and receive a new batch of messages.
3.  **Group Management**: If the command fails with a `NOGROUP` error, the `auto_new` function is called to automatically create the consumer group, making setup seamless.
4.  **Parse and Process**: All claimed and new messages are parsed by `parse_stream` into a list of `StreamItem`s.
5.  **Concurrent Execution with Zero-Copy**:
    -   The system uses `async_scoped` to spawn a `tokio` task for each `StreamItem`.
    -   Crucially, this allows the tasks to borrow data (like the message body) directly from the `StreamItem` without needing to clone it (`'static` lifetime is not required).
    -   This "zero-copy" approach significantly reduces memory overhead and improves performance, especially for large messages.
6.  **Error Handling & Retry**:
    -   If the `run` method returns an error, the message will be retried later.
    -   The `retry` count (delivery count) for each message is tracked and passed to the `run` method. If a message's retry count exceeds `max_retry`, it is passed to the `on_error` callback of the `Parse` trait for final handling (e.g., moving to a dead-letter queue).
7.  **Message Chaining**: If the `run` method returns `Ok(Some(new_kv))`, the new message is added to the queue using `XADD`, enabling workflow chaining.
8.  **Cleanup**: Successfully processed messages (or those handled by `on_error`) are acknowledged and deleted from the stream using `rm_id_li` (`XACK` and `XDEL`) to prevent reprocessing.

## Tech Stack

-   **Rust**: Core language for performance and safety.
-   **Tokio**: Asynchronous runtime for handling concurrency.
-   **Async Scoped**: Enables spawning non-`'static` futures, allowing for efficient zero-copy processing of borrowed data.
-   **Fred**: A high-performance, low-level Redis client for Rust.
-   **ThisError**: A library for deriving boilerplate `Error` implementations.

## Directory Structure

-   `src/lib.rs`: The library's main entry point. It exports the public API, including the `Parse` trait and key structs like `Conf`, `ReadGroup`, and `StreamItem`.
-   `src/conf.rs`: Defines the `Conf` struct, which centralizes all configuration parameters.
-   `src/read_group.rs`: Contains the core consumer logic within the `ReadGroup` struct and its `run` method.
-   `src/auto_new.rs`: Provides the `auto_new` function to automatically create a stream consumer group.
-   `src/parse_stream.rs`: Includes utilities for parsing responses from `XREADGROUP` and `XAUTOCLAIM`.
-   `src/rm_id_li.rs`: A helper function to `XACK` (acknowledge) and `XDEL` (delete) processed messages.
-   `src/error.rs`: Defines custom error types for the application.
-   `tests/`: Integration tests demonstrating usage patterns.

## API Reference

### `Conf`

A struct to hold all configuration for the `ReadGroup` consumer.

-   `stream`: The Redis key for the stream.
-   `group`: The consumer group name.
-   `consumer`: A unique name for this consumer.
-   `block_ms`: The time in milliseconds to block waiting for new messages.
-   `claim_idle_ms`: The idle time in milliseconds after which a pending message is considered abandoned and can be claimed by another consumer.
-   `count`: The maximum number of messages to fetch in a single batch.
-   `max_retry`: The maximum number of times a message will be retried before being passed to the `on_error` handler.

### `Parse` Trait

A trait that defines the application logic for message handling. You must implement this trait.

-   `run(&self, kv: &Kv, retry: u64) -> impl Future<Output = aok::Result<Option<Kv>>> + Send`:
    -   The asynchronous method called to process a single message.
    -   `kv`: The message data, as a `Vec<(Bytes, Bytes)>`.
    -   `retry`: The current retry count (delivery count) for this message.
    -   Return `Ok(None)` on successful processing.
    -   Return `Ok(Some(new_kv))` to add a new message to the queue (for workflow chaining).
    -   Return `Err(...)` to retry the message later.
-   `on_error(&self, kv: Kv, error: String) -> impl Future<Output = aok::Void> + Send`:
    -   The asynchronous method called when a message has failed more than `max_retry` times.
    -   `kv`: The message data that failed.
    -   `error`: The last error message that caused the failure.

### `ReadGroup`

The main consumer struct.

-   `ReadGroup::new(parse: P, conf: Conf)`: Creates a new `ReadGroup` instance.
-   `run(&self)`: Starts the infinite processing loop.

### `StreamItem`

Represents a single message from the Redis Stream.

-   `id`: The unique ID of the message.
-   `retry`: The delivery count (number of times delivered).
-   `idle_ms`: Time in milliseconds the message has been idle (if claimed).
-   `kv`: The message payload as a vector of Key-Value byte pairs.

## History

In 1983, Vivek Ranadive, a 26-year-old MIT graduate, observed that while hardware components communicated via a "bus", software lacked a similar standard mechanism. He envisioned a "software bus" where applications could publish and subscribe to information without direct, rigid connections. This idea led to the creation of **The Information Bus (TIB)**, the first commercial message queue software. TIB revolutionised financial trading floors by replacing manual chalkboards with real-time digital streams, allowing different trading systems to communicate instantly. This innovation laid the groundwork for modern event-driven architectures and the message queue systems we rely on today, such as Redis Streams.

---

## About

This project is an open-source component of [js0.site ⋅ Refactoring the Internet Plan](https://js0.site).

We are redefining the development paradigm of the Internet in a componentized way. Welcome to follow us:

* [Google Group](https://groups.google.com/g/js0-site)
* [js0site.bsky.social](https://bsky.app/profile/js0site.bsky.social)

---

<a id="zh"></a>

# msgq : 基于 Redis Stream 的健壮消息队列

基于 Redis Stream 的健壮消息队列，支持自动认领和重试处理。

## 目录

- [功能特性](#功能特性)
- [使用演示](#使用演示)
- [设计思路](#设计思路)
- [技术堆栈](#技术堆栈)
- [目录结构](#目录结构)
- [API 参考](#api-参考)
- [历史背景](#历史背景)

## 功能特性

- **基于 Redis Stream**: 利用 `XREADGROUP` 实现高效、可扩展的消息消费。
- **消费组**: 支持组内多个消费者并行处理，提高吞吐量。
- **自动创建组**: 如果 Redis Stream 组和消费者不存在，会自动创建。
- **可靠投递**: 实现空闲消息（Pending Messages）的自动认领，防止消息丢失。
- **并发处理**: 使用 `tokio::spawn` 结合 `async-scoped` 并发处理消息。
- **零拷贝**: 利用 `async-scoped` 处理来自 Stream 响应的借用数据，无需克隆，从而最大化性能。
- **消息链**: `Parse::run` 方法可以返回 `Some(Kv)` 来向队列添加新消息，实现工作流链式处理。
- **可配置重试**: 为失败的消息提供自动重试机制，并可配置重试次数。
- **集中化配置**: 使用简单的 `Conf` 结构体管理所有连接和行为设置。
- **基于 Trait 的回调**: 使用 `Parse` Trait 定义消息处理和错误处理逻辑，使代码更清晰和可复用。

## 使用演示

定义一个结构体并实现 `Parse` trait，以处理您的消息和错误逻辑。

```rust
use msgq::{Conf, Kv, Parse, ReadGroup};
use std::future::Future;
use aok::{OK, Void};
use log::info;

// 1. 定义你的消息处理器

struct MyParser;

impl Parse for MyParser {
  // 2. 实现消息处理逻辑
  async fn run(&self, kv: &Kv, retry: u64) -> aok::Result<Option<Kv>> {
    info!("处理消息: {:?}, 重试次数: {}", kv, retry);
    // 返回 Ok(None) 表示成功处理
    // 返回 Ok(Some(new_kv)) 向队列添加新消息
    // 返回 Err(...) 稍后重试该消息
    Ok(None)
  }

  // 3. 为重试失败的消息实现错误处理逻辑
  async fn on_error(&self, kv: Kv, error: String) -> Void {
    info!("错误处理: {:?}, 错误: {}", kv, error);
    OK
  }
}

#[tokio::main]
async fn main() -> Void {
  // 4. 初始化环境 (例如使用 xboot 设置全局 Redis 客户端)
  // xboot::init().await?; 

  // 5. 配置消费者
  let conf = Conf::new(
    "s1", // stream 键名
    "g1", // 消费组名称
    "c1", // 消费者名称
    5,    // block_sec: 等待新消息的阻塞时间（秒）
    60,   // claim_idle_sec: 认领空闲消息的超时时间（秒）
    10,   // count: 单次获取的最大消息数
    3,    // max_retry: 最大重试次数
  );

  // 6. 创建 ReadGroup 并运行
  ReadGroup::new(MyParser, conf).run().await?;
  OK
}
```

## 设计思路

`ReadGroup::run` 方法执行一个持续的循环，确保消息处理的健壮性：

1.  **认领空闲消息**: 首先调用 `XPENDING` 查找空闲时间超过 `claim_idle_ms` 的消息，并使用 `XAUTOCLAIM` 认领它们。这确保了因消费者崩溃或缓慢而未处理的消息能够被重新处理。
2.  **获取新消息**: 接着执行带 `BLOCK` 超时的 `XREADGROUP` 命令，高效地等待并接收一批新消息。
3.  **组管理**: 如果命令因 `NOGROUP` 错误而失败，将调用 `auto_new` 函数自动创建消费组，使启动过程无缝衔接。
4.  **解析与处理**: 所有被认领和新获取的消息都由 `parse_stream` 解析成 `StreamItem` 列表。
5.  **零拷贝并发执行**:
    -   系统使用 `async_scoped` 为每个 `StreamItem` 生成一个 `tokio` 任务。
    -   关键在于，这允许任务直接从 `StreamItem` 借用数据（如消息体），而无需进行克隆（不需要 `'static` 生命周期）。
    -   这种"零拷贝"方法显著降低了内存开销并提高了性能，尤其是对于大消息而言。
6.  **错误处理与重试**:
    -   如果 `run` 方法返回错误，该消息将在稍后被重试。
    -   系统会跟踪每条消息的 `retry`（重试）次数并传递给 `run` 方法。如果消息的重试次数超过 `max_retry`，它将被传递给 `Parse` trait 的 `on_error` 回调进行最终处理（例如，移入死信队列）。
7.  **消息链**: 如果 `run` 方法返回 `Ok(Some(new_kv))`，新消息将使用 `XADD` 添加到队列，实现工作流链式处理。
8.  **清理**: 成功处理（或由 `on_error` 处理）的消息会通过 `rm_id_li`（`XACK` 和 `XDEL`）进行确认和删除，以防止重复处理。

## 技术堆栈

-   **Rust**: 核心语言，提供高性能和内存安全。
-   **Tokio**: 用于处理并发的异步运行时。
-   **Async Scoped**: 支持生成非 `'static` 的 Future，允许对借用数据进行高效的零拷贝处理。
-   **Fred**: 一个高性能、底层的 Rust Redis 客户端。
-   **ThisError**: 一个用于派生 `Error` 实现的库，简化错误处理。

## 目录结构

-   `src/lib.rs`: 库的主入口文件。它导出公共 API，包括 `Parse` trait 和核心结构体，如 `Conf`、`ReadGroup` 和 `StreamItem`。
-   `src/conf.rs`: 定义 `Conf` 结构体，用于集中管理所有配置参数。
-   `src/read_group.rs`: 在 `ReadGroup` 结构体及其 `run` 方法中实现核心的消费者逻辑。
-   `src/auto_new.rs`: 提供 `auto_new` 函数，用于自动创建 Stream 消费组。
-   `src/parse_stream.rs`: 包含用于解析 `XREADGROUP` 和 `XAUTOCLAIM` 响应的工具。
-   `src/rm_id_li.rs`: 一个辅助函数，用于 `XACK`（确认）和 `XDEL`（删除）已处理的消息。
-   `src/error.rs`: 定义应用程序的自定义错误类型。
-   `tests/`: 集成测试，展示了库的使用模式。

## API 参考

### `Conf`

一个用于保存 `ReadGroup` 消费者所有配置的结构体。

-   `stream`: Stream 的 Redis 键名。
-   `group`: 消费组的名称。
-   `consumer`: 当前消费者的唯一名称。
-   `block_ms`: 阻塞等待新消息的毫秒数。
-   `claim_idle_ms`: 一条待处理消息在被认领前可以空闲的毫秒数。超过此时限，消息可被其他消费者认领。
-   `count`: 单个批次中获取的最大消息数。
-   `max_retry`: 一条消息在被移交至 `on_error` 处理器前将尝试重试的最大次数。

### `Parse` Trait

一个定义了消息处理应用逻辑的 trait。你必须实现此 trait。

-   `run(&self, kv: &Kv, retry: u64) -> impl Future<Output = aok::Result<Option<Kv>>> + Send`:
    -   用于处理单条消息的异步方法。
    -   `kv`: 消息数据，类型为 `Vec<(Bytes, Bytes)>`。
    -   `retry`: 当前消息的重试次数（投递计数）。
    -   成功处理时返回 `Ok(None)`。
    -   返回 `Ok(Some(new_kv))` 向队列添加新消息（用于工作流链式处理）。
    -   返回 `Err(...)` 稍后重试该消息。
-   `on_error(&self, kv: Kv, error: String) -> impl Future<Output = aok::Void> + Send`:
    -   当一条消息的失败次数超过 `max_retry` 时调用的异步方法。
    -   `kv`: 失败的消息数据。
    -   `error`: 导致失败的最后一条错误信息。

### `ReadGroup`

主要的消费者结构体。

-   `ReadGroup::new(parse: P, conf: Conf)`: 创建一个新的 `ReadGroup` 实例。
-   `run(&self)`: 启动无限处理循环。

### `StreamItem`

表示来自 Redis Stream 的单条消息。

-   `id`: 消息唯一 ID。
-   `retry`: 投递次数（重试计数）。
-   `idle_ms`: 消息闲置时间（毫秒，若被认领）。
-   `kv`: 消息负载，为键值对字节向量。

## 历史背景

1983 年，26 岁的麻省理工学院毕业生 Vivek Ranadive 观察到，虽然硬件组件通过“总线”进行通信，但软件缺乏类似的标准机制。他设想了一种“软件总线”，应用程序可以在没有直接、僵化连接的情况下发布和订阅信息。这一想法促成了 **The Information Bus (TIB)** 的诞生，这是首款商业消息队列软件。TIB 取代了人工黑板，实现了实时数字流，彻底改变了金融交易大厅，允许不同的交易系统即时通信。这一创新为现代事件驱动架构以及我们今天所依赖的 Redis Stream 等消息队列系统奠定了基础。

---

## 关于

本项目为 [js0.site ⋅ 重构互联网计划](https://js0.site) 的开源组件。

我们正在以组件化的方式重新定义互联网的开发范式，欢迎关注：

* [谷歌邮件列表](https://groups.google.com/g/js0-site)
* [js0site.bsky.social](https://bsky.app/profile/js0site.bsky.social)
