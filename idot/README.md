[English](#en) | [中文](#zh)

---

<a id="en"></a>

# idot : DNS over TLS Client for Rust

Based on [idns](https://crates.io/crates/idns). See idns for `DnsRace`, `Cache`, `Parse` trait, and more.

## Features

- RFC 7858 compliant DoT implementation
- Built-in DoT server list (Cloudflare, Google, Quad9, Alibaba DNS)
- Async/await with Tokio
- TLS 1.3
- A, AAAA, MX, TXT, NS, CNAME, PTR, SRV record types
- Connection reuse
- 9s timeout

## Installation

```toml
[dependencies]
idot = "0.1"
idns = "0.1"
```

## Usage

### DnsRace + Cache (Recommended)

Race multiple servers and cache results:

```rust
use idot::{DOT_LI, dot_li};
use idns::{Cache, DnsRace, Mx, Query};
use std::time::Instant;

#[tokio::main]
async fn main() {
  let race = DnsRace::new(dot_li(DOT_LI));
  let cache: Cache<Mx> = Cache::new(60); // 60s TTL

  // First query (cache miss)
  let t1 = Instant::now();
  let r1 = cache.query(&race, "gmail.com").await;
  let d1 = t1.elapsed();
  println!("First: {}ms", d1.as_millis());
  if let Some(mx_list) = &*r1.unwrap() {
    for mx in mx_list {
      println!("  {} {}", mx.priority, mx.server);
    }
  }

  // Second query (cache hit)
  let t2 = Instant::now();
  let _ = cache.query(&race, "gmail.com").await;
  let d2 = t2.elapsed();
  println!("Cache: {}μs", d2.as_micros());
}
```

### Basic Query

```rust
use idot::{Dot, host_ip, QType};
use idns::Query;

#[tokio::main]
async fn main() {
  let client = Dot::new(host_ip("cloudflare-dns.com", 1, 1, 1, 1));

  if let Ok(Some(answers)) = client.answer_li(QType::A, "example.com").await {
    for a in answers {
      println!("{} TTL={}", a.val, a.ttl);
    }
  }
}
```

### Custom Server

```rust
use idot::{Dot, host_ip, QType};
use idns::Query;

#[tokio::main]
async fn main() {
  let server = host_ip("dns.google", 8, 8, 8, 8);
  let client = Dot::new(server);

  if let Ok(Some(answers)) = client.answer_li(QType::AAAA, "google.com").await {
    for a in answers {
      println!("{}", a.val);
    }
  }
}
```

## API Reference

### Structs

#### `Dot`

DoT client with connection reuse. Implements `idns::Query` trait.

#### `HostIp`

Server configuration with `host: SmolStr` (TLS SNI) and `ip: IpAddr`.

### Functions

- `host_ip(host, a, b, c, d) -> HostIp` - Create HostIp from hostname and IPv4
- `dot_li(li: &[HostIp]) -> Vec<Dot>` - Create Dot clients from HostIp list

### Constants

#### `DOT_LI`

Pre-configured DoT servers:

| Server          | IP                        |
| --------------- | ------------------------- |
| Cloudflare      | 1.1.1.1, 1.0.0.1          |
| Google          | 8.8.8.8, 8.8.4.4          |
| Quad9           | 9.9.9.9                   |
| 360 DNS (China) | 101.226.4.6, 218.30.118.6 |
| TWNIC (Taiwan)  | 101.101.101.101           |
| IIJ DNS (Japan) | 103.2.57.5                |

#### `dns` module

Server hostname constants:

- `dns::CLOUDFLARE` - `"cloudflare-dns.com"`
- `dns::GOOGLE` - `"dns.google"`
- `dns::QUAD9` - `"dns.quad9.net"`
- `dns::DNS360` - `"dot.360.cn"`
- `dns::TWNIC` - `"101.101.101.101"`
- `dns::IIJ` - `"public.dns.iij.jp"`
- `dns::ALIDNS` - `"dns.alidns.com"` (disabled: incomplete TXT records)
- `dns::DNSPOD` - `"dot.pub"` (disabled: connection issues)

## Architecture

```mermaid
graph TD
    A[Client] --> B[Dot.query]
    B --> C[conn]
    C --> D{Has conn?}
    D -->|Yes| E[Take]
    D -->|No| F[dial]
    F --> G[TCP + TLS 1.3]
    G --> I[TlsStream]
    E --> J[send]
    I --> J
    J --> K[DNS Message]
    K --> L[Response]
    L --> M[Parse]
    M --> N[Answers]
    N --> O[return_conn]
```

### Implementation Details

- Random DNS message ID (verified on response)
- 2-byte length prefix for framing (RFC 7858)
- EDNS OPT with 4096 byte payload
- `LazyLock` for TLS `ClientConfig`
- `RwLock<Option<TlsStream>>` for connection reuse
- TCP_NODELAY enabled
- 9s timeout

## Tech Stack

| Component | Library               |
| --------- | --------------------- |
| TLS       | rustls + tokio-rustls |
| Async     | tokio                 |
| Buffer    | bytes                 |
| Error     | thiserror             |
| DNS Parse | dns_parse             |

## DoT vs DoQ

| Feature               | DoT (idot) | DoQ (idoq) |
| --------------------- | ---------- | ---------- |
| Protocol              | TCP + TLS  | QUIC       |
| Port                  | 853        | 853        |
| Multiplexing          | No         | Yes        |
| 0-RTT                 | No         | Yes        |
| Head-of-line blocking | Yes        | No         |
| Maturity              | High       | Medium     |

---

## About

This project is an open-source component of [js0.site ⋅ Refactoring the Internet Plan](https://js0.site).

We are redefining the development paradigm of the Internet in a componentized way. Welcome to follow us:

- [Google Group](https://groups.google.com/g/js0-site)
- [js0site.bsky.social](https://bsky.app/profile/js0site.bsky.social)

---

<a id="zh"></a>

# idot : Rust DNS over TLS 客户端

基于 [idns](https://crates.io/crates/idns)。`DnsRace`、`Cache`、`Parse` trait 等更多功能请查看 idns。

## 特性

- 符合 RFC 7858 的 DoT 实现
- 内置 DoT 服务器列表 (Cloudflare、Google、Quad9、阿里 DNS)
- 基于 Tokio 异步
- TLS 1.3
- 支持 A、AAAA、MX、TXT、NS、CNAME、PTR、SRV 记录
- 连接复用
- 9 秒超时

## 安装

```toml
[dependencies]
idot = "0.1"
idns = "0.1"
```

## 使用

### DnsRace + Cache（推荐）

竞速查询多个服务器并缓存结果：

```rust
use idot::{DOT_LI, dot_li};
use idns::{Cache, DnsRace, Mx, Query};
use std::time::Instant;

#[tokio::main]
async fn main() {
  let race = DnsRace::new(dot_li(DOT_LI));
  let cache: Cache<Mx> = Cache::new(60); // 60 秒 TTL

  // 首次查询（缓存未命中）
  let t1 = Instant::now();
  let r1 = cache.query(&race, "gmail.com").await;
  let d1 = t1.elapsed();
  println!("首次: {}ms", d1.as_millis());
  if let Some(mx_list) = &*r1.unwrap() {
    for mx in mx_list {
      println!("  {} {}", mx.priority, mx.server);
    }
  }

  // 再次查询（缓存命中）
  let t2 = Instant::now();
  let _ = cache.query(&race, "gmail.com").await;
  let d2 = t2.elapsed();
  println!("缓存: {}μs", d2.as_micros());
}
```

### 基本查询

```rust
use idot::{Dot, host_ip, QType};
use idns::Query;

#[tokio::main]
async fn main() {
  let client = Dot::new(host_ip("cloudflare-dns.com", 1, 1, 1, 1));

  if let Ok(Some(answers)) = client.answer_li(QType::A, "example.com").await {
    for a in answers {
      println!("{} TTL={}", a.val, a.ttl);
    }
  }
}
```

### 自定义服务器

```rust
use idot::{Dot, host_ip, QType};
use idns::Query;

#[tokio::main]
async fn main() {
  let server = host_ip("dns.google", 8, 8, 8, 8);
  let client = Dot::new(server);

  if let Ok(Some(answers)) = client.answer_li(QType::AAAA, "google.com").await {
    for a in answers {
      println!("{}", a.val);
    }
  }
}
```

## API 参考

### 结构体

#### `Dot`

DoT 客户端，支持连接复用。实现 `idns::Query` trait。

#### `HostIp`

服务器配置，包含 `host: SmolStr`（TLS SNI）和 `ip: IpAddr`。

### 函数

- `host_ip(host, a, b, c, d) -> HostIp` - 从主机名和 IPv4 创建 HostIp
- `dot_li(li: &[HostIp]) -> Vec<Dot>` - 从 HostIp 列表创建 Dot 客户端

### 常量

#### `DOT_LI`

预配置 DoT 服务器：

| 服务器          | IP                        |
| --------------- | ------------------------- |
| Cloudflare      | 1.1.1.1, 1.0.0.1          |
| Google          | 8.8.8.8, 8.8.4.4          |
| Quad9           | 9.9.9.9                   |
| 360 DNS（中国） | 101.226.4.6, 218.30.118.6 |
| TWNIC（台湾）   | 101.101.101.101           |
| IIJ DNS（日本） | 103.2.57.5                |

#### `dns` 模块

服务器主机名常量：

- `dns::CLOUDFLARE` - `"cloudflare-dns.com"`
- `dns::GOOGLE` - `"dns.google"`
- `dns::QUAD9` - `"dns.quad9.net"`
- `dns::DNS360` - `"dot.360.cn"`
- `dns::TWNIC` - `"101.101.101.101"`
- `dns::IIJ` - `"public.dns.iij.jp"`
- `dns::ALIDNS` - `"dns.alidns.com"`（已禁用：TXT 记录不完整）
- `dns::DNSPOD` - `"dot.pub"`（已禁用：连接问题）

## 架构

```mermaid
graph TD
    A[客户端] --> B[Dot.query]
    B --> C[conn]
    C --> D{有连接?}
    D -->|是| E[取出]
    D -->|否| F[dial]
    F --> G[TCP + TLS 1.3]
    G --> I[TlsStream]
    E --> J[send]
    I --> J
    J --> K[DNS 消息]
    K --> L[响应]
    L --> M[解析]
    M --> N[应答]
    N --> O[return_conn]
```

### 实现细节

- 随机 DNS 消息 ID（响应时验证）
- 2 字节长度前缀分帧 (RFC 7858)
- EDNS OPT 4096 字节负载
- `LazyLock` 延迟初始化 TLS `ClientConfig`
- `RwLock<Option<TlsStream>>` 连接复用
- 启用 TCP_NODELAY
- 9 秒超时

## 技术栈

| 组件     | 库                    |
| -------- | --------------------- |
| TLS      | rustls + tokio-rustls |
| 异步     | tokio                 |
| 缓冲     | bytes                 |
| 错误     | thiserror             |
| DNS 解析 | dns_parse             |

## DoT vs DoQ

| 特性     | DoT (idot) | DoQ (idoq) |
| -------- | ---------- | ---------- |
| 协议     | TCP + TLS  | QUIC       |
| 端口     | 853        | 853        |
| 多路复用 | 否         | 是         |
| 0-RTT    | 否         | 是         |
| 队头阻塞 | 是         | 否         |
| 成熟度   | 高         | 中         |

---

## 关于

本项目为 [js0.site ⋅ 重构互联网计划](https://js0.site) 的开源组件。

我们正在以组件化的方式重新定义互联网的开发范式，欢迎关注：

- [谷歌邮件列表](https://groups.google.com/g/js0-site)
- [js0site.bsky.social](https://bsky.app/profile/js0site.bsky.social)
