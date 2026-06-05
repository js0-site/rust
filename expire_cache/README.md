[English](#en) | [中文](#zh)

---

<a id="en"></a>

# expire_cache: High-performance generational cache

`expire_cache` implements an efficient expiration cache using generational collection strategy. Instead of tracking individual item expiration times, it maintains two data buckets (generations), significantly reducing memory overhead and CPU usage for expiration checks.

## Features

- **High Performance**: O(1) amortized expiration overhead per item
- **Concurrent Access**: Built on [papaya](https://docs.rs/papaya) for lock-free thread-safe operations
- **Async Support**: Native async initialization with `get_or_init_async!`
- **Flexible Storage**: Support for both key-value maps and sets
- **Simple API**: Clean interface with `get`, `insert`, and initialization methods

## Installation

```toml
[dependencies]
expire_cache = { version = "0.1.22", features = ["hashmap", "get_or_init_async"] }
```

Available features:

- `hashmap`: Enable HashMap support (papaya::HashMap)
- `hashset`: Enable HashSet support (papaya::HashSet)
- `get_or_init`: Enable synchronous initialization
- `get_or_init_async`: Enable asynchronous initialization macro

## Quick Start

### Basic Usage

```rust
use expire_cache::Expire;
use papaya::HashMap;
use std::time::Duration;

#[tokio::main]
async fn main() {
  let cache: Expire<HashMap<&str, &str>> = Expire::new(60);

  cache.insert("key", "value");

  if let Some(val) = cache.get("key") {
    println!("Found: {val}");
  }

  // Wait for expiration
  tokio::time::sleep(Duration::from_secs(120)).await;
  assert!(cache.get("key").is_none());
}
```

### Async Initialization

```rust
use expire_cache::{Expire, get_or_init_async};
use papaya::HashMap;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
  let cache: Expire<HashMap<String, String>> = Expire::new(60);

  async fn load_data() -> Result<String, std::io::Error> {
    Ok("data_for_user_123".to_string())
  }

  let value: Result<_, std::io::Error> = get_or_init_async!(cache, "user_123", load_data);
  println!("Loaded: {}", value?);

  Ok(())
}
```

### Sync Initialization

```rust
use expire_cache::{Expire, GetOrInit};
use papaya::HashMap;

fn main() -> Result<(), std::io::Error> {
  let cache: Expire<HashMap<String, String>> = Expire::new(60);

  let value = cache.get_or_init("user_123", |key| {
    Ok::<_, std::io::Error>(format!("data_for_{key}"))
  })?;
  println!("Loaded: {value}");

  Ok(())
}
```

### Set Usage

```rust
use expire_cache::Expire;
use papaya::HashSet;

#[tokio::main]
async fn main() {
  let cache: Expire<HashSet<&str>> = Expire::new(60);

  cache.insert("active_session", ());

  if cache.get("active_session").is_some() {
    println!("Session exists");
  }
}
```

## API Reference

### `Expire<T: Map>`

- `new(expire: u64) -> Self`: Create cache with expiration period in seconds
- `get(&self, key) -> Option<RefVal>`: Retrieve value from cache
- `insert(&self, key, val)`: Insert value into cache
- `get_or_init(&self, key, func) -> Result<RefVal, E>`: Sync initialization (requires `get_or_init` feature)

### `get_or_init_async!` macro

- `get_or_init_async!(cache, key, init_fn) -> Result<Val, E>`: Async initialization macro, calls `init_fn()` only on cache miss

#### Why a macro instead of a callback function?

Rust's `impl Trait` return types create distinct opaque types for each call site. When a closure captures references and returns an `impl Future`, the compiler cannot unify the Future's lifetime with the closure parameter's lifetime in generic contexts. This causes "lifetime mismatch" errors.

The macro approach bypasses this by inlining the code at the call site, so the async expression is awaited directly without going through a generic callback, naturally avoiding the lifetime inference issues.

This is a known Rust limitation: [rust-lang/rust#100013](https://github.com/rust-lang/rust/issues/100013)

## Design

### Generational Collection

The cache uses a double-buffer approach with two generations:

1. **Insertion**: New entries always go to the active generation
2. **Lookup**: Check active generation first, then passive generation
3. **Expiration**: Background task periodically clears passive generation and swaps roles
4. **Lifecycle**: Items live between `expire` and `2 * expire` seconds

This approach trades absolute precision for significant throughput improvements and reduced memory fragmentation.

## License

MulanPSL-2.0

---

## About

This project is an open-source component of [js0.site ⋅ Refactoring the Internet Plan](https://js0.site).

We are redefining the development paradigm of the Internet in a componentized way. Welcome to follow us:

- [Google Group](https://groups.google.com/g/js0-site)
- [js0site.bsky.social](https://bsky.app/profile/js0site.bsky.social)

---

<a id="zh"></a>

# expire_cache: 高性能分代缓存

`expire_cache` 实现了基于分代收集策略的高效过期缓存。它不追踪单个条目的过期时间，而是维护两个数据桶（代），显著降低过期检查的内存开销和 CPU 使用率。

## 特性

- **高性能**：每条目摊销 O(1) 过期开销
- **并发访问**：基于 [papaya](https://docs.rs/papaya) 的无锁线程安全操作
- **异步支持**：原生异步初始化宏 `get_or_init_async!`
- **灵活存储**：支持键值映射和集合
- **简洁 API**：清晰的 `get`、`insert` 和初始化接口

## 安装

```toml
[dependencies]
expire_cache = { version = "0.1.22", features = ["hashmap", "get_or_init_async"] }
```

可用特性：

- `hashmap`：启用 HashMap 支持 (papaya::HashMap)
- `hashset`：启用 HashSet 支持 (papaya::HashSet)
- `get_or_init`：启用同步初始化
- `get_or_init_async`：启用异步初始化宏

## 快速开始

### 基本用法

```rust
use expire_cache::Expire;
use papaya::HashMap;
use std::time::Duration;

#[tokio::main]
async fn main() {
  let cache: Expire<HashMap<&str, &str>> = Expire::new(60);

  cache.insert("key", "value");

  if let Some(val) = cache.get("key") {
    println!("Found: {val}");
  }

  // 等待过期
  tokio::time::sleep(Duration::from_secs(120)).await;
  assert!(cache.get("key").is_none());
}
```

### 异步初始化

```rust
use expire_cache::{Expire, get_or_init_async};
use papaya::HashMap;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
  let cache: Expire<HashMap<String, String>> = Expire::new(60);

  async fn load_data() -> Result<String, std::io::Error> {
    Ok("data_for_user_123".to_string())
  }

  let value: Result<_, std::io::Error> = get_or_init_async!(cache, "user_123", load_data);
  println!("Loaded: {}", value?);

  Ok(())
}
```

### 同步初始化

```rust
use expire_cache::{Expire, GetOrInit};
use papaya::HashMap;

fn main() -> Result<(), std::io::Error> {
  let cache: Expire<HashMap<String, String>> = Expire::new(60);

  let value = cache.get_or_init("user_123", |key| {
    Ok::<_, std::io::Error>(format!("data_for_{key}"))
  })?;
  println!("Loaded: {value}");

  Ok(())
}
```

### 集合用法

```rust
use expire_cache::Expire;
use papaya::HashSet;

#[tokio::main]
async fn main() {
  let cache: Expire<HashSet<&str>> = Expire::new(60);

  cache.insert("active_session", ());

  if cache.get("active_session").is_some() {
    println!("Session exists");
  }
}
```

## API 参考

### `Expire<T: Map>`

- `new(expire: u64) -> Self`：创建带过期周期的缓存（秒）
- `get(&self, key) -> Option<RefVal>`：从缓存检索值
- `insert(&self, key, val)`：向缓存插入值
- `get_or_init(&self, key, func) -> Result<RefVal, E>`：同步初始化（需要 `get_or_init` feature）

### `get_or_init_async!` 宏

- `get_or_init_async!(cache, key, init_fn) -> Result<Val, E>`：异步初始化宏，仅在缓存未命中时调用 `init_fn()`

#### 为什么用宏而不是回调函数？

Rust 的 `impl Trait` 返回类型在每个调用点会生成不同的 opaque type。当闭包捕获引用并返回 `impl Future` 时，编译器无法在泛型上下文中统一 Future 的生命周期与闭包参数的生命周期，导致"生命周期不匹配"错误。

宏通过在调用点内联代码来绕过这个问题，异步表达式直接被 await，无需经过泛型回调，自然避免了生命周期推断问题。

这是 Rust 的已知限制：[rust-lang/rust#100013](https://github.com/rust-lang/rust/issues/100013)

## 设计

### 分代收集策略

缓存使用双缓冲方法，包含两个代：

1. **插入**：新条目总是进入活跃代
2. **查找**：先检查活跃代，再检查被动代
3. **过期**：后台任务定期清空被动代并交换角色
4. **生命周期**：条目存活时间在 `expire` 到 `2 * expire` 秒之间

这种方法牺牲了绝对精度，换取了显著的吞吐量提升和内存碎片减少。

## 许可证

MulanPSL-2.0

---

## 关于

本项目为 [js0.site ⋅ 重构互联网计划](https://js0.site) 的开源组件。

我们正在以组件化的方式重新定义互联网的开发范式，欢迎关注：

- [谷歌邮件列表](https://groups.google.com/g/js0-site)
- [js0site.bsky.social](https://bsky.app/profile/js0site.bsky.social)
