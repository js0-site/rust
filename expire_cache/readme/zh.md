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