[English](#en) | [中文](#zh)

---

<a id="en"></a>

# smtp_srv : High-Performance SMTPS Server with Auto-Refreshing Certificates

## Table of Contents
- [Introduction](#introduction)
- [Features](#features)
- [Architecture](#architecture)
- [Usage](#usage)
- [Exported API](#exported-api)
- [Tech Stack](#tech-stack)
- [Directory Structure](#directory-structure)
- [History](#history)

## Introduction

`smtp_srv` is an asynchronous SMTPS server built with Rust, designed for high-performance mail handling with Redis/Kvrocks backend.

Core capabilities:
- Port 25: Receives mail from external MTAs and forwards based on rules
- Port 465: Accepts authenticated user connections via implicit TLS for sending mail

The server automatically refreshes TLS certificates based on hostname TLD, ensuring secure and uninterrupted service.

## Features

- **Auto-Refreshing TLS**: Certificates fetched and updated automatically by hostname TLD
- **Dual-Port Architecture**: Port 25 for receiving/forwarding, Port 465 for authenticated sending
- **Dynamic Forwarding**: Real-time rule lookup from Redis/Kvrocks
- **DKIM Signing**: Integrated DKIM support for outgoing mail
- **Graceful Shutdown**: Safe termination via system signal handling
- **High Throughput**: Built on Tokio async runtime

## Architecture

### Port 25 - Mail Reception & Forwarding

External MTAs connect to port 25 to deliver mail. STARTTLS is optional. The server queries forwarding rules from Redis and routes mail accordingly.

```mermaid
graph TD
  MTA[External MTA] -->|1. Port 25| P25[smtp_srv :25]
  P25 -->|2. STARTTLS Optional| TLS{TLS?}
  TLS -->|Yes| Cert[Cert Module]
  Cert --> CBH[cert_by_host]
  TLS -->|No| RCPT[RCPT TO]
  CBH --> RCPT
  RCPT -->|3. Query| Fwd[Forward Module]
  Fwd -->|4. mailForward:host| DB[(Redis/Kvrocks)]
  DB -->|5. Target| DATA[DATA]
  DATA -->|6. Forward| Mailer[Mailer Module]
  Mailer -->|7. smtp_send| Target[Target Server]
```

### Port 465 - User Authentication & Sending

Users connect to port 465 with implicit TLS, authenticate via SMTP AUTH, and send mail through the server.

```mermaid
graph TD
  User[Mail Client] -->|1. Port 465| P465[smtp_srv :465]
  P465 -->|2. Implicit TLS| Cert[Cert Module]
  Cert --> CBH[cert_by_host]
  CBH -->|3. TLS Established| Auth[AUTH LOGIN]
  Auth -->|4. Verify| DB[(Redis/Kvrocks)]
  DB -->|5. Auth OK| Data[DATA]
  Data -->|6. Send| Mailer[Mailer Module]
  Mailer -->|7. DKIM Sign| SMTP[smtp_send]
  SMTP -->|8. Deliver| Target[Recipient Server]
```

### Forwarding Rule Lookup

Redis stores forwarding rules in hash format:
- Key: `mailForward:<domain>`
- Field: username or `*` (wildcard)
- Value: target email address

Lua scripts (`mailForward`, `mailForwardSet`) handle single and batch lookups with wildcard fallback.

## Usage

Add dependency:

```toml
[dependencies]
smtp_srv = "0.2.24"
```

Entry point:

```rust
use aok::{OK, Void};
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[static_init::constructor(0)]
extern "C" fn _init() {
  log_init::init();
}

#[tokio::main]
async fn main() -> Void {
  xboot::init().await?;
  let _ = rustls::crypto::ring::default_provider().install_default();
  smtp_srv::run().await;
  OK
}
```

Run:

```bash
cargo run --release
```

Test sending (requires environment variables `SMTP_USER` and `SMTP_PASSWORD`):

```javascript
import nodemailer from "nodemailer";

const SMTP = nodemailer.createTransport({
  host: "127.0.0.1",
  port: 465,
  secure: true,
  auth: {
    user: process.env.SMTP_USER,
    pass: process.env.SMTP_PASSWORD,
  },
  tls: {
    servername: "smtp.example.com",
  },
});

await SMTP.sendMail({
  from: '"Sender" <sender@example.com>',
  to: "recipient@example.com",
  subject: "Test",
  text: "Hello",
});
```

## Exported API

### Functions

- `run()`: Async entry point. Initializes server with `Forward`, `AuthEnv`, `Mailer`, and `Cert` implementations, then awaits shutdown signal.

### Structs

- `Cert`: Implements `ssl_trait::CertByHost`. Resolves certificates by normalizing hostname to TLD.

- `Mailer`: Implements `smtp_recv::Mailer`. Handles mail delivery via `smtp_send` with DKIM signing. Provides `send()` for authenticated user mail and `forward()` for forwarded mail.

### Modules

- `r`: Constants for Redis function names (`MAIL_FORWARD`, `MAIL_FORWARD_SET`).

## Tech Stack

| Component | Technology |
|-----------|------------|
| Runtime | [Tokio](https://tokio.rs/) |
| Language | Rust (Edition 2024) |
| Database | Redis / [Kvrocks](https://kvrocks.apache.org/) |
| TLS | [rustls](https://github.com/rustls/rustls) |
| Redis Client | [fred](https://github.com/aembke/fred.rs) |
| Allocator | [mimalloc](https://github.com/microsoft/mimalloc) |
| Core | `smtp_recv`, `smtp_send`, `cert_by_host` |

## Directory Structure

```
smtp_srv/
├── src/
│   ├── lib.rs        # Library exports, run()
│   ├── main.rs       # Application entry
│   ├── cert.rs       # TLS certificate resolution
│   ├── forward.rs    # Mail forwarding logic
│   ├── mailer.rs     # Mail sending with DKIM
│   └── r.rs          # Redis function constants
├── lua/
│   └── mailForward.lua  # Redis Lua scripts
└── test/
    └── test_smtp.js     # SMTP client test
```

## History

The `@` symbol in email addresses was chosen by Ray Tomlinson in 1971 when he sent the first network email on ARPANET. He needed a character to separate username from hostname that wouldn't appear in names. Looking at his Model 33 Teletype keyboard, he picked `@` — a symbol rarely used at the time. The content of that first email was likely just test characters like "QWERTYUIOP". This simple choice became the universal identifier for digital communication.

SMTP itself was formalized in RFC 821 (1982) by Jonathan Postel. The protocol has evolved through multiple RFCs, with port 465 originally assigned for SMTPS in 1997, deprecated, then re-standardized in RFC 8314 (2018) for implicit TLS submission.

---

## About

This project is an open-source component of [js0.site ⋅ Refactoring the Internet Plan](https://js0.site).

We are redefining the development paradigm of the Internet in a componentized way. Welcome to follow us:

* [Google Group](https://groups.google.com/g/js0-site)
* [js0site.bsky.social](https://bsky.app/profile/js0site.bsky.social)

---

<a id="zh"></a>

# smtp_srv : 高性能自动热更新证书的 SMTPS 服务器

## 目录
- [简介](#简介)
- [功能特性](#功能特性)
- [架构设计](#架构设计)
- [使用演示](#使用演示)
- [API 接口](#api-接口)
- [技术栈](#技术栈)
- [目录结构](#目录结构)
- [历史趣闻](#历史趣闻)

## 简介

`smtp_srv` 是基于 Rust 构建的异步 SMTPS 服务器，使用 Redis/Kvrocks 作为后端存储，专为高性能邮件处理设计。

核心能力：
- 25 端口：接收外部 MTA 邮件，按规则转发
- 465 端口：用户通过隐式 TLS 认证登录后发送邮件

服务器根据主机名 TLD 自动刷新 TLS 证书，确保服务安全且不中断。

## 功能特性

- **证书自动热更新**：根据主机名 TLD 自动获取和更新证书
- **双端口架构**：25 端口收信/转发，465 端口认证发信
- **动态转发规则**：从 Redis/Kvrocks 实时查询转发配置
- **DKIM 签名**：集成发件 DKIM 签名支持
- **优雅停机**：监听系统信号，安全终止服务
- **高吞吐量**：基于 Tokio 异步运行时

## 架构设计

### 25 端口 - 收信与转发

外部 MTA 连接 25 端口投递邮件，STARTTLS 可选。服务器从 Redis 查询转发规则后路由邮件。

```mermaid
graph TD
  MTA[外部 MTA] -->|1. 25 端口| P25[smtp_srv :25]
  P25 -->|2. STARTTLS 可选| TLS{TLS?}
  TLS -->|是| Cert[Cert 证书模块]
  Cert --> CBH[cert_by_host]
  TLS -->|否| RCPT[RCPT TO]
  CBH --> RCPT
  RCPT -->|3. 查询| Fwd[Forward 转发模块]
  Fwd -->|4. mailForward:host| DB[(Redis/Kvrocks)]
  DB -->|5. 目标| DATA[DATA]
  DATA -->|6. 转发| Mailer[Mailer 投递模块]
  Mailer -->|7. smtp_send| Target[目标服务器]
```

### 465 端口 - 用户认证与发信

用户通过 465 端口隐式 TLS 连接，经 SMTP AUTH 认证后发送邮件。

```mermaid
graph TD
  User[邮件客户端] -->|1. 465 端口| P465[smtp_srv :465]
  P465 -->|2. 隐式 TLS| Cert[Cert 证书模块]
  Cert --> CBH[cert_by_host]
  CBH -->|3. TLS 建立| Auth[AUTH LOGIN]
  Auth -->|4. 验证| DB[(Redis/Kvrocks)]
  DB -->|5. 认证通过| Data[DATA]
  Data -->|6. 发送| Mailer[Mailer 投递模块]
  Mailer -->|7. DKIM 签名| SMTP[smtp_send]
  SMTP -->|8. 投递| Target[收件服务器]
```

### 转发规则查询

Redis 以 Hash 格式存储转发规则：
- Key：`mailForward:<域名>`
- Field：用户名或 `*`（通配符）
- Value：目标邮箱地址

Lua 脚本（`mailForward`、`mailForwardSet`）处理单条和批量查询，支持通配符回退。

## 使用演示

添加依赖：

```toml
[dependencies]
smtp_srv = "0.2.24"
```

入口文件：

```rust
use aok::{OK, Void};
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[static_init::constructor(0)]
extern "C" fn _init() {
  log_init::init();
}

#[tokio::main]
async fn main() -> Void {
  xboot::init().await?;
  let _ = rustls::crypto::ring::default_provider().install_default();
  smtp_srv::run().await;
  OK
}
```

运行：

```bash
cargo run --release
```

发信测试（需设置环境变量 `SMTP_USER` 和 `SMTP_PASSWORD`）：

```javascript
import nodemailer from "nodemailer";

const SMTP = nodemailer.createTransport({
  host: "127.0.0.1",
  port: 465,
  secure: true,
  auth: {
    user: process.env.SMTP_USER,
    pass: process.env.SMTP_PASSWORD,
  },
  tls: {
    servername: "smtp.example.com",
  },
});

await SMTP.sendMail({
  from: '"发件人" <sender@example.com>',
  to: "recipient@example.com",
  subject: "测试",
  text: "你好",
});
```

## API 接口

### 函数

- `run()`：异步入口函数。使用 `Forward`、`AuthEnv`、`Mailer`、`Cert` 实现初始化服务器，等待停机信号。

### 数据结构

- `Cert`：实现 `ssl_trait::CertByHost`。将主机名规范化为 TLD 后解析证书。

- `Mailer`：实现 `smtp_recv::Mailer`。通过 `smtp_send` 处理邮件投递，支持 DKIM 签名。提供 `send()` 处理认证用户发信，`forward()` 处理转发邮件。

### 模块

- `r`：Redis 函数名常量（`MAIL_FORWARD`、`MAIL_FORWARD_SET`）。

## 技术栈

| 组件 | 技术 |
|------|------|
| 运行时 | [Tokio](https://tokio.rs/) |
| 编程语言 | Rust (Edition 2024) |
| 数据库 | Redis / [Kvrocks](https://kvrocks.apache.org/) |
| TLS | [rustls](https://github.com/rustls/rustls) |
| Redis 客户端 | [fred](https://github.com/aembke/fred.rs) |
| 内存分配器 | [mimalloc](https://github.com/microsoft/mimalloc) |
| 核心组件 | `smtp_recv`, `smtp_send`, `cert_by_host` |

## 目录结构

```
smtp_srv/
├── src/
│   ├── lib.rs        # 库导出，run()
│   ├── main.rs       # 应用入口
│   ├── cert.rs       # TLS 证书解析
│   ├── forward.rs    # 邮件转发逻辑
│   ├── mailer.rs     # DKIM 签名发信
│   └── r.rs          # Redis 函数常量
├── lua/
│   └── mailForward.lua  # Redis Lua 脚本
└── test/
    └── test_smtp.js     # SMTP 客户端测试
```

## 历史趣闻

电子邮件中的 `@` 符号由 Ray Tomlinson 于 1971 年选定，当时他在 ARPANET 上发送了第一封网络邮件。他需要找到能区分用户名和主机名的字符，且不会出现在人名中。看着 Model 33 电传打字机键盘，他选中了 `@` —— 当时鲜少使用的符号。那封邮件的内容可能只是 "QWERTYUIOP" 之类的测试字符。这个简单的选择成为了数字通信的通用标识。

SMTP 协议由 Jonathan Postel 在 RFC 821（1982）中正式定义。协议历经多次 RFC 演进，465 端口最初于 1997 年分配给 SMTPS，后被废弃，又在 RFC 8314（2018）中重新标准化为隐式 TLS 提交端口。

---

## 关于

本项目为 [js0.site ⋅ 重构互联网计划](https://js0.site) 的开源组件。

我们正在以组件化的方式重新定义互联网的开发范式，欢迎关注：

* [谷歌邮件列表](https://groups.google.com/g/js0-site)
* [js0site.bsky.social](https://bsky.app/profile/js0site.bsky.social)
