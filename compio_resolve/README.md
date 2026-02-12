[English](#en) | [中文](#zh)

---

<a id="en"></a>

# compio-resolve : Zero-cost Async DNS Resolver with Cache

`compio-resolve` provides a high-performance, asynchronous DNS resolver for the `compio` ecosystem. It integrates seamlessly with `compio-net` to provide custom DNS resolution capabilities, replacing the default thread-pool based implementation with a true async approach.

## ✨ Features

- **Zero-Cost Abstraction**: Uses `extern "Rust"` FFI to plug into `compio-net` without dynamic dispatch (`dyn`).
- **Async Native**: Built on `compio` runtime, efficient and non-blocking.
- **Smart Caching**: Built-in LRU cache (enabled by default) for high-speed repeated lookups.
- **System Integrated**: Reads `/etc/hosts` and `/etc/resolv.conf` (on Unix) for correct resolution.

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
compio-resolve = "0.1"
```

**Note**: Adding this crate will automatically enable the `extern_resolve` feature in `compio-net`, causing `compio` to use this resolver instead of the default one.

## 🚀 Usage

**CRITICAL**: You MUST explicitly import `compio_resolve` in your crate root (`lib.rs` or `main.rs`) to ensure the linker includes the resolver symbols.

```rust
// main.rs or lib.rs
// ⚠️ Must import compio_resolve to ensure symbols are linked
extern crate compio_resolve;

fn main() {
    compio::task::block_on(async {
        let stream = compio::net::TcpStream::connect("google.com:80").await.unwrap();
        // ... use stream
    });
}
```

If you don't do this, the linker might discard `compio_resolve` as "unused", leading to a "symbol not found" error or fallback to the (disabled) default resolver depending on configuration.

## 🧩 Architecture

The following diagram illustrates how `compio-resolve` integrates with `compio-net`:

```mermaid
graph TD
    UserCode[User Code] -->|Connect| CompioNet[compio-net]
    CompioNet -->|Resolve| ExternResolve(extern_resolve mechanism)
    ExternResolve -.->|FFI Call| CompioResolve[compio-resolve]
    
    subgraph compio-resolve
        Cache[LRU Cache]
        Hosts["/etc/hosts"]
        ResolvConf["/etc/resolv.conf"]
        UDPSocket[Async UDP Socket]
        
        CompioResolve -->|Check| Hosts
        CompioResolve -->|Check| Cache
        Cache -->|Hit| Return[Return IPs]
        Cache -->|Miss| Query[Query Nameserver]
        Query -->|Read Config| ResolvConf
        Query -->|Send Packet| UDPSocket
        UDPSocket -->|Response| UpdateCache[Update Cache]
        UpdateCache --> Return
    end
```

## 🛠️ Tech Stack

- **Runtime**: `compio-runtime`
- **Networking**: `compio-net` (UDP/TCP)
- **Caching**: `scc` (Scalable Concurrent Containers) for high-performance concurrent LRU cache.
- **Parsing**: `logos` (lexer), `zerocopy` (zero-copy parsing).

## 📜 History & Trivia

In the early days of `compio`, DNS resolution relied on `spawn_blocking` to call the synchronous `getaddrinfo` syscall (similar to `tokio`). While robust, this consumed thread pool resources. `compio-resolve` was born from the desire to implement a pure-Rust, fully async resolver that could be swapped in at compile time with zero runtime overhead—achieved through some clever FFI tricks and compile-time assertions!

---

## About

This project is an open-source component of [js0.site ⋅ Refactoring the Internet Plan](https://js0.site).

We are redefining the development paradigm of the Internet in a componentized way. Welcome to follow us:

* [Google Group](https://groups.google.com/g/js0-site)
* [js0site.bsky.social](https://bsky.app/profile/js0site.bsky.social)

---

<a id="zh"></a>

# compio-resolve : 零开销带缓存的异步 DNS 解析器

`compio-resolve` 为 `compio` 生态系统提供了一个高性能的异步 DNS 解析器。它能无缝集成到 `compio-net` 中，替换默认的基于线程池的实现，提供真正的全异步解析能力。

## ✨ 特性

- **零开销抽象**：使用 `extern "Rust"` FFI 接入 `compio-net`，无动态分发（无 `dyn`）。
- **原生异步**：基于 `compio` 运行时构建，高效非阻塞。
- **智能缓存**：内置 LRU 缓存（默认启用），显著加速重复查询。
- **系统集成**：正确读取 `/etc/hosts` 和 `/etc/resolv.conf` (Unix) 以符合系统行为。

## 📦 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
compio-resolve = "0.1"
```

**注意**：引入此 crate 会自动启用 `compio-net` 的 `extern_resolve` 特性，使 `compio` 在链接时使用本解析器。

## 🚀 使用方法

**重要**：你必须在 crate 根文件（`lib.rs` 或 `main.rs`）中显式引入 `compio_resolve`，以确保链接器包含解析器符号。

```rust
// main.rs 或 lib.rs
// ⚠️ 必须引入 compio_resolve，确保 resolve_set! 生成的符号被链接
extern crate compio_resolve;

fn main() {
    compio::task::block_on(async {
        // 无需修改代码，compio::net 会自动使用 compio-resolve
        let stream = compio::net::TcpStream::connect("google.com:80").await.unwrap();
        // ... 使用 stream
    });
}
```

如果不这样做，链接器可能会将 `compio_resolve` 视为“未使用”而丢弃，导致符号未找到错误。

## 🧩 架构设计

下图展示了 `compio-resolve` 如何与 `compio-net` 交互：

```mermaid
graph TD
    UserCode[用户代码] -->|连接| CompioNet[compio-net]
    CompioNet -->|解析| ExternResolve(extern_resolve 机制)
    ExternResolve -.->|FFI 调用| CompioResolve[compio-resolve]
    
    subgraph compio-resolve
        Cache[LRU 缓存]
        Hosts["/etc/hosts"]
        ResolvConf["/etc/resolv.conf"]
        UDPSocket[异步 UDP Socket]
        
        CompioResolve -->|检查| Hosts
        CompioResolve -->|检查| Cache
        Cache -->|命中| Return[返回 IP]
        Cache -->|未命中| Query[查询 Nameserver]
        Query -->|读取配置| ResolvConf
        Query -->|发送报文| UDPSocket
        UDPSocket -->|响应| UpdateCache[更新缓存]
        UpdateCache --> Return
    end
```

## 🛠️ 技术栈

- **运行时**: `compio-runtime`
- **网络**: `compio-net` (UDP/TCP)
- **缓存**: `scc` (Scalable Concurrent Containers) 实现高性能并发 LRU 缓存。
- **解析**: `logos` (词法分析), `zerocopy` (零拷贝解析)。

## 📜 历史与趣闻

在 `compio` 早期，DNS 解析依赖 `spawn_blocking` 调用同步的 `getaddrinfo` 系统调用（类似 `tokio`）。虽然稳健，但这占用了线程池资源。`compio-resolve` 诞生于对极致性能的追求——通过纯 Rust 实现全异步解析，并通过巧妙的 FFI 技巧和编译时断言，实现了编译期替换且零运行时开销的插件化设计！

---

## 关于

本项目为 [js0.site ⋅ 重构互联网计划](https://js0.site) 的开源组件。

我们正在以组件化的方式重新定义互联网的开发范式，欢迎关注：

* [谷歌邮件列表](https://groups.google.com/g/js0-site)
* [js0site.bsky.social](https://bsky.app/profile/js0site.bsky.social)
