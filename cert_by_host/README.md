[English](#en) | [中文](#zh)

---

<a id="en"></a>

# cert_by_host : Dynamic SSL Certificate Loading with Auto-Expiration

`cert_by_host` is a high-performance Rust library designed to dynamically load HTTPS certificates from Kvrocks (a Redis-compatible store) based on hostnames. It features an efficient in-memory cache with automatic expiration handling, ensuring your application serves the correct certificates with minimal latency.

## Table of Contents

- [Features](#features)
- [Usage](#usage)
- [Design Architecture](#design-architecture)
- [Tech Stack](#tech-stack)
- [Project Structure](#project-structure)
- [API Reference](#api-reference)
- [The Story](#the-story)

## Features

- **Dynamic Loading**: Fetches SSL certificates on-demand from Kvrocks using the hostname as the key.
- **High-Performance Caching**: Utilizes `DashMap` for concurrent, lock-free reads, ensuring extremely fast access during TLS handshakes.
- **Automatic Expiration**: A background task periodically cleans up expired certificates, preventing memory leaks and ensuring validity.
- **Cache Penetration Protection**: Remembers non-existent hosts for a short period to avoid repeated database hits for invalid domains.
- **Rustls Integration**: Directly returns `rustls` compatible types (`PrivateKeyDer`, `CertificateDer`), ready for immediate use.

## Usage

Add `cert_by_host` to your `Cargo.toml`. Ensure you have a Kvrocks/Redis instance running and configured.

```rust
use cert_by_host::get;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the system (spawns background tasks)
    xboot::init().await?;

    let host = "example.com";

    // Retrieve the certificate configuration
    if let Some(ssl_config_ref) = get(host).await? {
        println!("Loaded certificate for: {}", host);
        // Use ssl_config_ref.key and ssl_config_ref.cert with your TLS acceptor
    } else {
        println!("No certificate found for: {}", host);
    }

    Ok(())
}
```

## Design Architecture

The library employs a multi-layered strategy to balance performance and freshness:

1.  **L1 Cache (Memory)**: When `get(host)` is called, it first checks a global `DashMap`. If present, it returns a reference immediately.
2.  **Negative Cache**: If the host is known to not exist (checked via `ExpireSet`), it returns `None` immediately to protect the backend.
3.  **L2 Storage (Kvrocks)**: If not in memory, it queries Kvrocks.
    -   **Data Format**: Expects a JSON string containing the private key and certificate chain in PEM format.
    -   **Parsing**: Parses the PEM data into `rustls` types and extracts the expiration date using `x509_parser`.
4.  **Expiration Management**:
    -   Upon successful load, the expiration timestamp is added to a `BTreeMap` (sorted by time).
    -   A background loop runs every hour. It efficiently queries the `BTreeMap` for deadlines that have passed and removes the corresponding entries from the main `DashMap`.

## Tech Stack

-   **Core**: Rust
-   **Async Runtime**: `tokio`
-   **Caching**: `dashmap` (concurrent hash map), `expire_set`
-   **TLS**: `rustls`, `rustls-pemfile`
-   **Database**: `fred` (Redis client), `xkv` wrapper
-   **Serialization**: `sonic_rs` (fast JSON)
-   **Time**: `coarsetime`
-   **Certificate Parsing**: `x509_parser`

## Project Structure

```
.
├── src
│   ├── lib.rs            # Core logic: caching, expiration task, public API
│   ├── get_by_kvrocks.rs # Database interaction and certificate parsing
│   └── error.rs          # Custom error definitions
├── tests
│   └── main.rs           # Integration tests
└── Cargo.toml
```

## API Reference

### `pub async fn get(host: impl Into<String>) -> Result<Option<Ref<'static, String, SslConfig>>>`

The main entry point.
-   **Input**: A hostname string (e.g., "example.com").
-   **Output**: A `Result` containing an `Option`.
    -   `Some(Ref<...>)`: A reference to the cached `SslConfig`. The `Ref` type allows concurrent read access.
    -   `None`: No certificate found for this host.

### `pub struct SslConfig`

Holds the parsed cryptographic material.
-   `pub key: PrivateKeyDer<'static>`: The private key.
-   `pub cert: Vec<CertificateDer<'static>>`: The certificate chain.

## The Story

In the era of SaaS and PaaS, platforms often manage tens of thousands of custom domains for their users. Loading all certificates into memory at startup is inefficient and slow. `cert_by_host` was born out of the need to serve SSL certificates dynamically and instantly. By combining the speed of in-memory caching with the persistence of Kvrocks, it solves the "C10K" problem for SSL termination, ensuring that even with millions of domains, your server only holds what's currently active in memory.

---

## About

This project is an open-source component of [js0.site ⋅ Refactoring the Internet Plan](https://js0.site).

We are redefining the development paradigm of the Internet in a componentized way. Welcome to follow us:

* [Google Group](https://groups.google.com/g/js0-site)
* [js0site.bsky.social](https://bsky.app/profile/js0site.bsky.social)

---

<a id="zh"></a>

# cert_by_host : 基于域名动态加载与自动过期的 HTTPS 证书管理

`cert_by_host` 是一个高性能 Rust 库，专为根据主机名从 Kvrocks（兼容 Redis 协议）动态加载 HTTPS 证书而设计。它内置了高效的内存缓存和自动过期处理机制，确保您的应用能以极低的延迟获取正确的证书。

## 目录

- [功能特性](#功能特性)
- [使用演示](#使用演示)
- [设计思路](#设计思路)
- [技术堆栈](#技术堆栈)
- [目录结构](#目录结构)
- [API 说明](#api-说明)
- [项目背景](#项目背景)

## 功能特性

- **动态加载**：按需通过主机名作为键值从 Kvrocks 获取 SSL 证书。
- **高性能缓存**：使用 `DashMap` 实现并发无锁读取，极大降低 TLS 握手时的延迟。
- **自动过期**：后台任务定期清理已过期的证书，防止内存泄漏并确保证书有效性。
- **防缓存穿透**：对不存在的主机名进行短时记录，避免无效请求频繁击穿至数据库。
- **Rustls 集成**：直接返回兼容 `rustls` 的类型 (`PrivateKeyDer`, `CertificateDer`)，可即插即用。

## 使用演示

在 `Cargo.toml` 中添加 `cert_by_host`。请确保您已配置并运行了 Kvrocks 或 Redis 实例。

```rust
use cert_by_host::get;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化系统（启动后台过期清理任务）
    xboot::init().await?;

    let host = "example.com";

    // 获取证书配置
    if let Some(ssl_config_ref) = get(host).await? {
        println!("已加载证书: {}", host);
        // 可直接使用 ssl_config_ref.key 和 ssl_config_ref.cert 配置 TLS
    } else {
        println!("未找到证书: {}", host);
    }

    Ok(())
}
```

## 设计思路

本项目采用多级策略来平衡性能与数据新鲜度：

1.  **一级缓存 (内存)**：调用 `get(host)` 时，优先查询全局 `DashMap`。若命中，直接返回引用。
2.  **负面缓存**：若已知该主机名不存在（通过 `ExpireSet` 检查），立即返回 `None`，保护后端存储。
3.  **二级存储 (Kvrocks)**：若内存未命中，则查询 Kvrocks。
    -   **数据格式**：期望存储格式为包含 PEM 格式私钥和证书链的 JSON 字符串。
    -   **解析处理**：将 PEM 数据解析为 `rustls` 类型，并使用 `x509_parser` 提取过期时间。
4.  **过期管理**：
    -   加载成功后，将过期时间戳存入按时间排序的 `BTreeMap`。
    -   后台循环每小时运行一次，高效查询 `BTreeMap` 中已过期的条目，并从主 `DashMap` 中移除。

## 技术堆栈

-   **核心语言**: Rust
-   **异步运行时**: `tokio`
-   **缓存机制**: `dashmap` (并发哈希表), `expire_set`
-   **TLS 支持**: `rustls`, `rustls-pemfile`
-   **数据库交互**: `fred` (Redis 客户端), `xkv` (封装层)
-   **序列化**: `sonic_rs` (高性能 JSON)
-   **时间处理**: `coarsetime`
-   **证书解析**: `x509_parser`

## 目录结构

```
.
├── src
│   ├── lib.rs            # 核心逻辑：缓存定义、过期任务、公共 API
│   ├── get_by_kvrocks.rs # 数据库交互与证书解析逻辑
│   └── error.rs          # 自定义错误定义
├── tests
│   └── main.rs           # 集成测试
└── Cargo.toml
```

## API 说明

### `pub async fn get(host: impl Into<String>) -> Result<Option<Ref<'static, String, SslConfig>>>`

核心入口函数。
-   **输入**: 主机名字符串 (如 "example.com")。
-   **输出**: 包含 `Option` 的 `Result`。
    -   `Some(Ref<...>)`: 指向缓存中 `SslConfig` 的引用。`Ref` 类型支持并发读取。
    -   `None`: 未找到该主机的证书。

### `pub struct SslConfig`

封装了解析后的加密材料。
-   `pub key: PrivateKeyDer<'static>`: 私钥。
-   `pub cert: Vec<CertificateDer<'static>>`: 证书链。

## 项目背景

在 SaaS 和 PaaS 平台盛行的今天，服务商往往需要管理成千上万个用户自定义域名。传统的做法是在启动时将所有证书加载到内存，这既低效又占用资源。`cert_by_host` 应运而生，旨在解决海量域名的 SSL 证书动态服务问题。通过结合内存缓存的极速体验与 Kvrocks 的持久化存储能力，它完美解决了 SSL 终端的 "C10K" 问题，确保即便面对百万级域名，您的服务器也仅需按需加载活跃资源。

---

## 关于

本项目为 [js0.site ⋅ 重构互联网计划](https://js0.site) 的开源组件。

我们正在以组件化的方式重新定义互联网的开发范式，欢迎关注：

* [谷歌邮件列表](https://groups.google.com/g/js0-site)
* [js0site.bsky.social](https://bsky.app/profile/js0site.bsky.social)
