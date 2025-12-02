# expire_cache : 高效双缓冲过期缓存

## 目录
- 简介
- 特性
- 使用演示
- 设计思路
- 技术堆栈
- 目录结构
- API 参考
- 历史小故事

## 简介
`expire_cache` 实现了高性能、并发安全的自动过期缓存。它采用双缓冲策略管理对象生命周期，确保内存使用高效，并在访问时实现零成本的过期检查。

## 特性
- **高性能**：基于 `dashmap` 或 `dashset` 实现并发访问。
- **低开销**：双缓冲机制消除了逐项检查时间戳的开销。
- **内存高效**：内部状态使用单次分配；批量清理过期项。
- **线程安全**：利用原子操作和 `unsafe` 优化指针访问，全线程安全。
- **异步支持**：基于 Tokio 运行时的后台定时任务处理过期。

## 使用演示
在 `Cargo.toml` 中添加：
```toml
[dependencies]
expire_cache = "0.1"
```

示例代码：
```rust
use std::time::Duration;
use expire_cache::Expire;
use dashmap::DashMap;

#[tokio::main]
async fn main() {
    // 初始化缓存，过期时间 5 秒
    let cache: Expire<DashMap<String, String>> = Expire::new(5);

    cache.insert(&"key".to_string(), "value".to_string());

    if let Some(val) = cache.get(&"key".to_string()) {
        println!("Found: {}", val);
    }

    tokio::time::sleep(Duration::from_secs(6)).await;

    // 项目已过期
    assert!(cache.get(&"key".to_string()).is_none());
}
```

## 设计思路
核心机制依赖于两个底层 Map（缓冲区）和一个原子索引。

1.  **结构**：`Inner` 结构体持有 `[T; 2]`（两个 Map）和 `AtomicUsize`（索引）。
2.  **写入**：总是写入由原子索引指向的当前活跃缓冲区。
3.  **读取**：优先检查活跃缓冲区；若未命中，检查非活跃缓冲区。这保证了数据至少在一个完整的过期周期内可用。
4.  **过期**：后台 Tokio 任务每隔 `expire` 秒唤醒一次。它切换原子索引，实质上交换了活跃/非活跃缓冲区，并清空新的活跃缓冲区（其中包含最旧的数据）。

## 技术堆栈
- **Rust**：核心语言。
- **Tokio**：用于后台过期任务的异步运行时。
- **DashMap**：高性能并发哈希表。
- **Atomic/Unsafe**：用于无锁状态管理和优化的内存布局。

## 目录结构
```
.
├── Cargo.toml
├── readme
│   ├── en.md
│   └── zh.md
├── src
│   ├── lib.rs      # 核心逻辑及 Expire 结构体
│   ├── map.rs      # DashMap 的 Map trait 实现
│   ├── set.rs      # DashSet 的 Map trait 实现
└── tests
    └── main.rs     # 集成测试
```

## API 参考

### `Expire<T>`
管理缓存状态的主结构体。`T` 必须实现 `Map` trait。

#### `new(expire: u64) -> Self`
创建新缓存实例。`expire` 指定轮转间隔（秒）。

#### `insert(&self, key: &T::Key, val: T::Val)`
将键值对插入活跃缓存。

#### `get(&self, key: &T::Key) -> Option<T::Val>`
获取值。先检查活跃缓存，后检查非活跃缓存。

### `Map` Trait
底层存储（如 `DashMap`, `DashSet`）的抽象接口。

## 历史小故事
“分代缓存”或“缓存轮转”的概念与硬件设计及垃圾回收策略中的智慧不谋而合。早期的主机系统如 IBM System/360 引入缓存以弥合 CPU 与内存的速度差异。随着时间推移，策略从简单的 LRU（最近最少使用）演变为更复杂的分代方法。

在硬件领域，“缓存衰减（Cache Decay）”技术通过关闭长时间（一个“代”）未被访问的缓存行来降低功耗。与之类似，`expire_cache` 将时间区间视为“代”。通过批量丢弃整代数据，它避免了追踪单个项目时间戳的巨大开销——这一技术让人联想到分代垃圾回收器，它们假设“年轻”对象往往早夭，因此可以批量回收。这种方法牺牲了绝对的精度（精确的过期时间），换取了巨大的吞吐量提升和内存碎片减少。