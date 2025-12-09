[English](#en) | [中文](#zh)

---

<a id="en"></a>

# smtp_srv : High-performance SMTPS server with auto-refreshing certificates via Redis / Kvrocks

> [!IMPORTANT]
> Configure the following environment variables before deployment. Ensure sensitive keys are protected.

```bash
# DKIM Configuration
DKIM_SK="B3H-XxXXxxXXxXxx"
DKIM_PREFIX="js0-rsa"

# SMTP Authentication (loaded via auth_env)
SMTP_PASSWORD=XxXXXXX
SMTP_USER=i@js0.site
```

## Table of Contents

- [Introduction](#introduction)
- [Features](#features)
- [Architecture](#architecture)
- [Tech Stack](#tech-stack)
- [Directory Structure](#directory-structure)
- [API Reference](#api-reference)
- [Usage & Demo](#usage--demo)
- [Email History: The Rise of Authentication](#email-history-the-rise-of-authentication)

## Introduction

`smtp_srv` is a robust SMTPS server implementation written in Rust, designed for reliability and zero-downtime maintenance. Its standout capability is the integration with [Redis](https://redis.io/) / [Kvrocks](https://kvrocks.apache.org/) for dynamic SSL certificate management. Certificates are automatically loaded and refreshed from the database before expiration, eliminating the need to restart the server when certificates update.

This project serves as the concrete implementation layer, binding the core `smtp_recv` engine with specific storage and authentication strategies.

## Features

*   **Zero-Downtime Certificate Rotation**: Leverages `crate::smtp_recv` to automatically load SSL/TLS certificates from Redis / Kvrocks.
*   **Auto-Refresh**: Monitors certificate validity and fetches fresh certificates seamlessly.
*   **High Performance**: Built on top of the asynchronous `tokio` runtime and efficient `mimalloc` memory allocator.
*   **Secure Defaults**: Enforces SMTPS (465) and integrates strict authentication flows.
*   **DKIM Signing**: Automated DKIM signature injection for outgoing mail reliability.

## Architecture

The system is designed with a modular "Receive-Process-Forward" pipeline:

1.  **Transport Layer**: `smtp_recv` handles the raw SMTP protocol state machine.
2.  **Certificate Provider**: `Cert` struct (implementing `ssl_trait::CertByHost`) delegates to `cert_by_host`. It queries Redis / Kvrocks based on the incoming connection's SNI (Server Name Indication), extracting the Top Level Domain (TLD) if necessary to find the matching certificate.
3.  **Authentication**: `AuthEnv` loads SMTP credentials, securing the relay access.
4.  **Message Handling**: Authenticated messages are passed to `Mailer`.
5.  **Delivery**: `Mailer` uses `smtp_send` to dispatch the email to its final destination, signing it with DKIM keys loaded from environment variables.

## Tech Stack

*   **Runtime**: `tokio` (Async I/O)
*   **Core Engine**: `smtp_recv` (SMTP Protocol implementation)
*   **Storage/Certificates**: `redis / kvrocks`, `cert_by_host`
*   **Cryptography**: `rustls` (Modern, safe TLS library)
*   **Memory Management**: `mimalloc` (High-performance allocator)
*   **Utilities**: `aok` (Error handling), `genv` (Environment parsing)

## Directory Structure

*   `src/lib.rs`: Library entry point, exports core modules.
*   `src/main.rs`: Application entry point, initializes runtime and global allocator.
*   `src/cert.rs`: Implements dynamic certificate retrieval from Redis / Kvrocks.
*   `src/mailer.rs`: Implements the mail delivery logic, connecting reception to transmission.
*   `test/`: Contains integration tests demonstrating usage with Node.js clients (`nodemailer`).

## API Reference

The library exposes the following key components:

### `Cert`
A zero-sized struct implementing `ssl_trait::CertByHost`.
*   **Functionality**: Intercepts TLS handshake requests, parses the hostname/TLD, and retrieves the corresponding active certificate from Redis / Kvrocks.

### `Mailer`
The email processing agent implementing `smtp_recv::Mailer`.
*   **Functionality**: Receives `UserMail` objects (containing the authenticated user ID and raw email content) and forwards them using the configured `smtp_send` transport.

### `run(port: u16) -> Void`
The main server loop.
*   **Signature**: `async fn run(port: u16) -> Void`
*   **Usage**: Starts the SMTPS server on the specified port, injecting the `AuthEnv`, `Mailer`, and `Cert` providers.

## Usage & Demo

See `@tests/` for a complete example using `nodemailer`.

To run the server locally:
```bash
# Ensure environment variables are set (see top of document)
cargo run --release
```

Test script excerpt (`test/test_smtp.js`):
```javascript
const SMTP = nodemailer.createTransport({
  host: "127.0.0.1",
  port: 465,
  secure: true, // Uses SMTPS
  auth: { user: SMTP_USER, pass: SMTP_PASSWORD },
  tls: { servername: "smtp.js0.site" }, // Trigger SNI for cert loading
});
```

## Email History: The Rise of Authentication

In the early days of ARPANET, email was a trusting system. Protocol designers in the 1980s didn't anticipate the modern era of spam and spoofing. By the early 2000s, this trust was broken. 

The response was a fragmented evolution of authentication standards. **DomainKeys** (from Yahoo!) and **Identified Internet Mail** (from Cisco) were competing approaches to verify sender identity. In 2004, these giants merged their efforts, creating **DKIM (DomainKeys Identified Mail)**.

DKIM brought cryptographic signatures to email headers, allowing receiving servers to verify that the email truly originated from the claimed domain and hadn't been tampered with. It was a pivotal moment, shifting email from a "best effort" delivery system to a verifiable trust network, laying the groundwork for the modern DMARC policies that protect inboxes today. This server implements these standards to ensure your transactional emails are trusted by recipients.

---

## About

This project is an open-source component of [js0.site ⋅ Refactoring the Internet Plan](https://js0.site).

We are redefining the development paradigm of the Internet in a componentized way. Welcome to follow us:

* [Google Group](https://groups.google.com/g/js0-site)
* [js0site.bsky.social](https://bsky.app/profile/js0site.bsky.social)

---

<a id="zh"></a>

# smtp_srv : 基于 Redis / Kvrocks 自动热更新证书的高性能 SMTPS 服务器

> [!IMPORTANT]
> 部署前请务必配置以下环境变量，并注意保护私钥安全。

```bash
# DKIM 配置
DKIM_SK="d8S-XxxxxxXXxXXxxXXxXxxxxx"
DKIM_PREFIX="js0-rsa"

# SMTP 认证 (通过 auth_env 加载)
SMTP_PASSWORD=XxXXXXX
SMTP_USER=i@js0.site
```

## 目录

- [简介](#简介)
- [功能特性](#功能特性)
- [架构设计](#架构设计)
- [技术栈](#技术栈)
- [目录结构](#目录结构)
- [API 参考](#api-参考)
- [使用演示](#使用演示)
- [邮件简史：认证的崛起](#邮件简史认证的崛起)

## 简介

`smtp_srv` 是用 Rust 编写的高可靠 SMTPS 服务器实现，专为零停机维护设计。其核心亮点不仅在于高性能，更在于与 [Redis](https://redis.io/) / [Kvrocks](https://kvrocks.apache.org/) 的深度集成，实现了 SSL 证书的动态管理。服务器能自动从数据库加载证书，并在证书过期前自动捕获更新，全过程无需重启服务。

本项目作为具体实现层，将核心的 `smtp_recv` 引擎与特定的存储 backend 和认证策略进行了绑定。

## 功能特性

*   **证书零停机热更**：利用 `crate::smtp_recv` 自动从 Redis / Kvrocks 加载 SSL/TLS 证书。
*   **自动刷新**：实时监控证书有效性，在过期前无缝获取新证书。
*   **极致性能**：基于 `tokio` 异步运行时和 `mimalloc` 内存分配器构建，高并发下表现优异。
*   **安全默认**：强制开启 SMTPS (465端口)，并集成严格的身份认证流程。
*   **DKIM 签名**：自动为出站邮件注入 DKIM 签名，提升邮件送达率和可信度。

## 架构设计

系统采用模块化的 “接收-处理-转发” 管道设计：

1.  **传输层**：`smtp_recv` 负责处理底层的 SMTP 协议状态机。
2.  **证书提供者**：`Cert` 结构体（实现 `ssl_trait::CertByHost`）对接 `cert_by_host`。它根据入站连接的 SNI (Server Name Indication) 解析主机名或顶级域名 (TLD)，从 Redis / Kvrocks 查询匹配的证书。
3.  **身份认证**：`AuthEnv` 加载 SMTP 凭据，确保仅授权用户可使用中继服务。
4.  **消息处理**：通过认证的邮件被传递给 `Mailer`。
5.  **投递**：`Mailer` 调用 `smtp_send` 将邮件发往最终目的地，并使用环境变量中的密钥进行 DKIM 签名。

## 技术栈

*   **运行时**: `tokio` (异步 I/O)
*   **核心引擎**: `smtp_recv` (SMTP 协议实现)
*   **存储/证书**: `redis / kvrocks`, `cert_by_host`
*   **加密**: `rustls` (现代、安全的 TLS 库)
*   **内存管理**: `mimalloc` (高性能分配器)
*   **工具库**: `aok` (错误处理), `genv` (环境变量解析)

## 目录结构

*   `src/lib.rs`: 库入口，导出核心模块。
*   `src/main.rs`: 应用入口，初始化运行时和全局分配器。
*   `src/cert.rs`: 实现从 Redis / Kvrocks 动态检索证书的逻辑。
*   `src/mailer.rs`: 实现邮件投递逻辑，连接“接收”与“发送”环节。
*   `test/`: 包含使用 `nodemailer` 的 Node.js 集成测试示例。

## API 参考

本库导出了以下核心组件：

### `Cert`
实现 `ssl_trait::CertByHost` 的零大小结构体。
*   **功能**: 拦截 TLS 握手请求，解析域名，并从 Redis / Kvrocks 获取对应的活跃证书。

### `Mailer`
实现 `smtp_recv::Mailer` 的邮件处理代理。
*   **功能**: 接收 `UserMail` 对象（包含已认证的用户 ID 和原始邮件内容），并使用配置好的 `smtp_send` 传输层进行转发。

### `run(port: u16) -> Void`
服务器主循环。
*   **签名**: `async fn run(port: u16) -> Void`
*   **用途**: 在指定端口启动 SMTPS 服务器，并注入 `AuthEnv`、`Mailer` 和 `Cert` 提供者。

## 使用演示

完整示例请参考 `@tests/` 目录下的 `nodemailer` 脚本。

本地运行：
```bash
# 确保已设置环境变量（见文档开头）
cargo run --release
```

测试脚本片段 (`test/test_smtp.js`):
```javascript
const SMTP = nodemailer.createTransport({
  host: "127.0.0.1",
  port: 465,
  secure: true, // 使用 SMTPS
  auth: { user: SMTP_USER, pass: SMTP_PASSWORD },
  tls: { servername: "smtp.js0.site" }, // 触发 SNI 证书加载
});
```

## 邮件简史：认证的崛起

在 ARPANET 的早期岁月，电子邮件系统建立在完全信任的基础上。80年代的协议设计者未曾预料到垃圾邮件和伪造身份的泛滥。到了21世纪初，这种纯粹的信任链条已濒临崩溃。

作为回应，技术界诞生了一系列碎片化的认证标准。Yahoo! 推出了 **DomainKeys**，Cisco 提出了 **Identified Internet Mail**，两者都在试图解决发送者身份验证的问题。2004年，这两大巨头并未分道扬镳，而是选择合并各自的成果，共同缔造了 **DKIM (DomainKeys Identified Mail)**。

DKIM 将加密签名引入邮件头，使接收方服务器能够验证邮件确实源自声称的域名且未被篡改。这是一个里程碑式的时刻，它将电子邮件从一个“尽力而为”的投递系统转变为一个可验证的信任网络，为如今保护我们收件箱的 DMARC 策略奠定了基石。本服务器正是实现了这些标准，确您的交易邮件能赢得收件人的信任。

---

## 关于

本项目为 [js0.site ⋅ 重构互联网计划](https://js0.site) 的开源组件。

我们正在以组件化的方式重新定义互联网的开发范式，欢迎关注：

* [谷歌邮件列表](https://groups.google.com/g/js0-site)
* [js0site.bsky.social](https://bsky.app/profile/js0site.bsky.social)
