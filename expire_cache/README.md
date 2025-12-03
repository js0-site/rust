[English](#en) | [中文](#zh)

---

<a id="en"></a>

# expire_cache

Efficient double-buffered expiration cache.

`expire_cache` implements a high-performance cache with expiration based on a "double-buffer" (or generational) strategy. Instead of tracking the expiration time of each individual item, it maintains two buckets (generations) of data. This approach significantly reduces memory overhead and CPU usage for expiration checks, making it ideal for high-throughput scenarios where precise expiration timing is not critical.

## Features

- **High Performance**: Uses a double-buffer strategy for O(1) expiration overhead per item (amortized). No background scanning of all items.
- **Concurrent**: Built on top of `DashMap` for high concurrency.
- **Async Support**: Supports `get_or_init_async` for asynchronous value initialization.
- **Flexible**: Supports both Key-Value cache (`DashMap`) and Set cache (`DashSet`).
- **Simple API**: Easy to use `get`, `insert`, `get_or_init`.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
expire_cache = "0.1"
```

To enable specific features:

```toml
[dependencies]
expire_cache = { version = "0.1", features = ["dashmap", "get_or_init_async"] }
```

Available features:
- `dashmap`: Enable `DashMap` support (default).
- `dashset`: Enable `DashSet` support.
- `get_or_init`: Enable synchronous `get_or_init`.
- `get_or_init_async`: Enable asynchronous `get_or_init_async`.

## Usage

### Basic Usage (Map)

```rust
use std::time::Duration;
use expire_cache::Expire;
use dashmap::DashMap;

#[tokio::main]
async fn main() {
    // Create a cache with a 60-second expiration cycle
    let cache: Expire<DashMap<String, String>> = Expire::new(60);

    cache.insert("key".to_string(), "value".to_string());

    if let Some(val) = cache.get("key") {
        println!("Found: {}", *val);
    }
}
```

### Async Initialization (`get_or_init_async`)

```rust
use expire_cache::Expire;
use dashmap::DashMap;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cache: Expire<DashMap<String, String>> = Expire::new(60);

    let val = cache
        .get_or_init_async("key", |_key| async {
            // Simulate async work, e.g., database query
            Ok("computed_value".to_string())
        })
        .await?;

    println!("Value: {}", *val);
    Ok(())
}
```

### Set Usage

```rust
use expire_cache::Expire;
use dashmap::DashSet;

#[tokio::main]
async fn main() {
    let set: Expire<DashSet<String>> = Expire::new(60);

    set.insert("item".to_string(), ());

    if set.get("item").is_some() {
        println!("Item exists");
    }
}
```

## How it Works

1.  **Double Buffering**: The cache maintains two underlying containers (e.g., `DashMap`), let's call them `A` and `B`.
2.  **Active & Passive**: At any time, one is "active" (receiving new inserts) and the other is "passive" (read-only, containing older data).
3.  **Reads**: `get` checks the active container first. If not found, it checks the passive container.
4.  **Writes**: `insert` always writes to the active container.
5.  **Rotation**: Every `expire` seconds, a background task clears the passive container and swaps the roles of `A` and `B`. The previously active container becomes passive (preserving its data for one more cycle), and the cleared container becomes the new active one.

This means an item will live for at least `expire` seconds and at most `2 * expire` seconds.

## License

MulanPSL-2.0

---

## About

This project is an open-source component of [js0.site ⋅ Refactoring the Internet Plan](https://js0.site).

We are redefining the development paradigm of the Internet in a componentized way. Welcome to follow us:

* [Google Group](https://groups.google.com/g/js0-site)
* [js0site.bsky.social](https://bsky.app/profile/js0site.bsky.social)

---

<a id="zh"></a>

# expire_cache

高效的双缓冲过期缓存。

`expire_cache` 实现了一种基于“双缓冲”（或分代）策略的高性能缓存。它不追踪单个条目的过期时间，而是维护两个数据桶（代）。这种方法显著降低了过期检查的内存开销和 CPU 使用率，非常适合对精确过期时间要求不高但追求高吞吐量的场景。

## 特性

- **高性能**：使用双缓冲策略，每条目的过期开销为摊销 O(1)。无需后台扫描所有条目。
- **并发安全**：基于 `DashMap` 构建，支持高并发访问。
- **异步支持**：支持 `get_or_init_async` 进行异步值初始化。
- **灵活**：支持键值缓存 (`DashMap`) 和集合缓存 (`DashSet`)。
- **简单 API**：易于使用的 `get`, `insert`, `get_or_init` 接口。

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
expire_cache = "0.1"
```

启用特定特性：

```toml
[dependencies]
expire_cache = { version = "0.1", features = ["dashmap", "get_or_init_async"] }
```

可用特性：
- `dashmap`: 启用 `DashMap` 支持（默认）。
- `dashset`: 启用 `DashSet` 支持。
- `get_or_init`: 启用同步 `get_or_init`。
- `get_or_init_async`: 启用异步 `get_or_init_async`。

## 使用方法

### 基本用法 (Map)

```rust
use std::time::Duration;
use expire_cache::Expire;
use dashmap::DashMap;

#[tokio::main]
async fn main() {
    // 创建一个过期周期为 60 秒的缓存
    let cache: Expire<DashMap<String, String>> = Expire::new(60);

    cache.insert("key".to_string(), "value".to_string());

    if let Some(val) = cache.get("key") {
        println!("Found: {}", *val);
    }
}
```

### 异步初始化 (`get_or_init_async`)

```rust
use expire_cache::Expire;
use dashmap::DashMap;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cache: Expire<DashMap<String, String>> = Expire::new(60);

    let val = cache
        .get_or_init_async("key", |_key| async {
            // 模拟异步工作，例如数据库查询
            Ok("computed_value".to_string())
        })
        .await?;

    println!("Value: {}", *val);
    Ok(())
}
```

### 集合用法 (Set)

```rust
use expire_cache::Expire;
use dashmap::DashSet;

#[tokio::main]
async fn main() {
    let set: Expire<DashSet<String>> = Expire::new(60);

    set.insert("item".to_string(), ());

    if set.get("item").is_some() {
        println!("Item exists");
    }
}
```

## 工作原理

1.  **双缓冲**：缓存内部维护两个底层容器（例如 `DashMap`），我们称之为 `A` 和 `B`。
2.  **活跃与被动**：在任何时候，其中一个是“活跃”的（接收新插入），另一个是“被动”的（只读，包含较旧的数据）。
3.  **读取**：`get` 操作首先检查活跃容器。如果未找到，再检查被动容器。
4.  **写入**：`insert` 总是写入活跃容器。
5.  **轮转**：每隔 `expire` 秒，后台任务会清空被动容器，并交换 `A` 和 `B` 的角色。之前的活跃容器变为被动容器（将其数据保留一个周期），而被清空的容器变为新的活跃容器。

这意味着一个条目的存活时间至少为 `expire` 秒，至多为 `2 * expire` 秒。

## 历史小故事

“分代缓存”或“缓存轮转”的概念与硬件设计及垃圾回收策略中的智慧不谋而合。早期的主机系统如 IBM System/360 引入缓存以弥合 CPU 与内存的速度差异。随着时间推移，策略从简单的 LRU（最近最少使用）演变为更复杂的分代方法。

在硬件领域，“缓存衰减（Cache Decay）”技术通过关闭长时间（一个“代”）未被访问的缓存行来降低功耗。与之类似，`expire_cache` 将时间区间视为“代”。通过批量丢弃整代数据，它避免了追踪单个项目时间戳的巨大开销——这一技术让人联想到分代垃圾回收器，它们假设“年轻”对象往往早夭，因此可以批量回收。这种方法牺牲了绝对的精度（精确的过期时间），换取了巨大的吞吐量提升和内存碎片减少。

## 许可证

MulanPSL-2.0

---

## 关于

本项目为 [js0.site ⋅ 重构互联网计划](https://js0.site) 的开源组件。

我们正在以组件化的方式重新定义互联网的开发范式，欢迎关注：

* [谷歌邮件列表](https://groups.google.com/g/js0-site)
* [js0site.bsky.social](https://bsky.app/profile/js0site.bsky.social)
