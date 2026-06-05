# expire_set : 基于非安全原始指针的高性能并发过期集合

基于 `expire_cache` 构建的高性能并发集合，使用非安全原始指针实现最大性能，支持自动过期。

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

它是 `expire_cache` 的封装，提供了简化的 `Set` 接口，同时利用了底层库高效的双缓冲过期策略。实现使用非安全原始指针在并发环境中实现最佳性能。

## 特性

- **极致省内存**：**不**为单个项目保存过期时间戳。
- **短时缓存利器**：非常适合"一分钟后过期"这类场景，如 404 洪水攻击防护或去重缓冲区。
- **高性能**：利用 `expire_cache` 的无锁和双缓冲机制，配合非安全原始指针实现最优性能。
- **高并发**：线程安全并发访问。
- **批量自动过期**：后台定时器轮转缓冲区，批量过期旧项目。

## 使用演示

在 `Cargo.toml` 中添加：

```toml
[dependencies]
expire_set = "0.1.7"
tokio = { version = "1", features = ["time", "rt"] }
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

核心设计依赖于 `expire_cache` 提供的 **双缓冲** 机制：

1.  **底层存储**：内部使用 `expire_cache::Expire<HashSet<K>>`（基于 Papaya）。
2.  **简化接口**：暴露专为 Set 操作定制的 `insert` 和 `contains` 方法。
3.  **高效过期**：继承了批量过期策略，按代清除数据而非逐个清除。

## API 文档

### `ExpireSet<K>`

主结构体。`K` 必须实现 `Hash + Eq + Send + Sync + 'static`。

#### `fn new(expire: u64) -> Self`

创建新 `ExpireSet`。

- `expire`: 缓冲区轮转间隔（秒）。项目存活时间约为 `expire` 到 `2 * expire` 秒。

#### `fn insert(&self, key: K)`

将键插入集合。

#### `fn contains<Q>(&self, key: &Q) -> bool`

检查键是否存在于集合中。

- `Q`: 可借用的查询类型
- 返回值: 如果键存在返回 `true`，否则返回 `false`

## 技术堆栈

- **Rust**: 核心语言。
- **expire_cache**: 底层过期逻辑。
- **Tokio**: 异步运行时。
- **Papaya**: 高性能并发哈希表（通过 `expire_cache`）。

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

本项目使用的"轮转缓存"技术类比于计算机图形学中的 **双缓冲**（Double Buffering）。

双缓冲起源于 20 世纪 60 年代末，并在 80 年代随着 **Amiga** 等系统的出现成为标准。在图形学中，它涉及在显示"前缓冲区"的同时向隐藏的"后缓冲区"绘图，然后瞬间交换两者以防止画面撕裂。

类似地，`expire_set`（通过 `expire_cache`）写入"当前"缓冲区，同时保留"上一个"缓冲区供读取。当定时器触发时，它"交换"缓冲区并清空旧缓冲区，确保平滑过渡和高效批量过期。
