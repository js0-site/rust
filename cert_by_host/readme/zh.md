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
- **高性能缓存**：使用 `papaya` 实现并发无锁读取，极大降低 TLS 握手时的延迟。
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

1.  **一级缓存 (内存)**：调用 `get(host)` 时，优先查询全局 `papaya::HashMap`。若命中，直接返回克隆。
2.  **负面缓存**：若已知该主机名不存在（通过 `ExpireSet` 检查），立即返回 `None`，保护后端存储。
3.  **二级存储 (Kvrocks)**：若内存未命中，则查询 Kvrocks。
    - **数据格式**：期望存储格式为包含 PEM 格式私钥和证书链的 JSON 字符串。
    - **解析处理**：将 PEM 数据解析为 `rustls` 类型，并使用 `x509_parser` 提取过期时间。
4.  **过期管理**：
    - 加载成功后，将过期时间戳存入按时间排序的 `BTreeMap`。
    - 后台循环每小时运行一次，高效查询 `BTreeMap` 中已过期的条目，并从主 `papaya::HashMap` 中移除。

## 技术堆栈

- **核心语言**: Rust
- **异步运行时**: `tokio`
- **缓存机制**: `papaya` (并发哈希表), `expire_set`
- **TLS 支持**: `rustls`, `rustls-pemfile`
- **数据库交互**: `fred` (Redis 客户端), `xkv` (封装层)
- **序列化**: `sonic_rs` (高性能 JSON)
- **时间处理**: `coarsetime`
- **证书解析**: `x509_parser`

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

### `pub async fn get(host: impl Into<String>) -> Result<Option<Cert>>`

核心入口函数。

- **输入**: 主机名字符串 (如 "example.com")。
- **输出**: 包含 `Option` 的 `Result`。
  - `Some(Cert)`: 包装 `Arc<SslConfig>` 的结构体，实现了 `Deref` 可直接访问。
  - `None`: 未找到该主机的证书。

### `pub struct SslConfig`

封装了解析后的加密材料。

- `pub key: PrivateKeyDer<'static>`: 私钥。
- `pub cert: Vec<CertificateDer<'static>>`: 证书链。

## 项目背景

在 SaaS 和 PaaS 平台盛行的今天，服务商往往需要管理成千上万个用户自定义域名。传统的做法是在启动时将所有证书加载到内存，这既低效又占用资源。`cert_by_host` 应运而生，旨在解决海量域名的 SSL 证书动态服务问题。通过结合内存缓存的极速体验与 Kvrocks 的持久化存储能力，它完美解决了 SSL 终端的 "C10K" 问题，确保即便面对百万级域名，您的服务器也仅需按需加载活跃资源。
