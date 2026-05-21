# set_timer : Rust 健壮的定时间隔执行库

> Rust 健壮的定时间隔执行库，支持基于 Tokio 的异步和同步闭包。

## 功能特性

- **双模式支持**：无缝支持同步和异步闭包的执行。
- **Tokio 集成**：基于强大的 `tokio` 运行时构建，确保高效的任务调度。
- **轻松取消**：返回 `JoinHandle`，允许立即且干净地中止任务。
- **熟悉接口**：API 设计致敬 JavaScript 的 `setInterval`，同时兼具 Rust 的安全性和高性能。

## 使用演示

在 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
set_timer = "0.1.0"
```

### 同步间隔执行

适用于简单的阻塞操作或快速计算：

```rust
use std::time::Duration;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use set_timer::set_timer;

#[tokio::main]
async fn main() {
    let count = Arc::new(AtomicUsize::new(0));
    let count_clone = count.clone();

    // 每 100ms 执行一次
    let handle = set_timer(
        move || {
            count_clone.fetch_add(1, Ordering::SeqCst);
            println!("Tick");
        },
        Duration::from_millis(100),
    );

    // 运行一段时间
    tokio::time::sleep(Duration::from_millis(350)).await;

    // 停止执行
    handle.abort();
}
```

### 异步间隔执行

适用于涉及 I/O、网络请求或其他异步任务的操作：

```rust
use std::time::Duration;
use set_timer::set_timer_async;

#[tokio::main]
async fn main() {
    let handle = set_timer_async(
        || async {
            // 在此执行异步工作
            println!("Async Tick");
        },
        Duration::from_millis(100),
    );

    tokio::time::sleep(Duration::from_millis(350)).await;
    handle.abort();
}
```

## 设计思路

本库利用了 Tokio 轻量级的任务生成系统。

1.  **任务生成**：调用 `set_timer` 或 `set_timer_async` 时，会使用 `tokio::spawn` 生成一个新的 Tokio 任务。
2.  **循环执行**：在任务内部，建立一个无限 `loop` 循环。
3.  **执行与等待**：
    - 执行提供的闭包（如果是异步则进行 `await`）。
    - 循环随后使用 `tokio::time::sleep` 暂停执行指定的 `period` 时长。
4.  **资源管理**：返回的 `JoinHandle` 作为控制机制。丢弃句柄不会停止任务，但在其上调用 `.abort()` 会立即终止循环，确保资源被释放。

## 技术堆栈

- **语言**：Rust (Edition 2024)
- **运行时**：Tokio (异步运行时, 时间工具)
- **测试**：Tokio Test, Aok

## 目录结构

```
.
├── Cargo.toml          # 项目配置及依赖
├── src/
│   └── lib.rs          # 核心库逻辑 (set_timer, set_timer_async)
└── tests/
    └── main.rs         # 集成测试及使用示例
```

## API 参考

### `set_timer`

```rust
pub fn set_timer<F>(func: F, period: Duration) -> JoinHandle<()>
where
    F: Fn() + Send + Sync + 'static
```

以固定的时间间隔 `period` 重复执行同步闭包 `func`。

### `set_timer_async`

```rust
pub fn set_timer_async<F, Fut>(func: F, period: Duration) -> JoinHandle<()>
where
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send
```

重复执行异步闭包 `func`。`period` 延迟会在 `func` 返回的 Future 完成后开始计算。

## 历史背景

`setInterval` 的概念源于 Web 开发的早期阶段。虽然通常与 JavaScript 联系在一起，但它实际上并不是 ECMAScript 核心规范的一部分。相反，它是作为 Web 浏览器（`Window` 接口）提供的“宿主环境”API 引入的，后来被 Node.js 采纳。

它成为了动态网页的基石，实现了从简单的数字时钟到复杂的轮询机制和动画等各种功能。本 crate 旨在将这一重要的实用工具带入 Rust 生态系统，并针对现代异步系统编程进行了适配。
