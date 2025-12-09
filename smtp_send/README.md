[English](#en) | [中文](#zh)

---

<a id="en"></a>

# smtp_send : Secure SMTP Email Sending with DKIM

`smtp_send` is a robust Rust library designed for sending emails via SMTP with built-in support for DKIM signing, automatic MX record lookups, and automatic rejection handling. It simplifies the process of sending authenticated emails, ensuring high deliverability and security.

## Table of Contents

- [Introduction](#introduction)
- [Features](#features)
- [Usage](#usage)
- [Design](#design)
- [Technology Stack](#technology-stack)
- [Directory Structure](#directory-structure)
- [API Reference](#api-reference)
- [History](#history)

## Introduction

Sending emails programmatically often involves complex configurations, especially when dealing with authentication standards like DKIM (DomainKeys Identified Mail) and finding the correct mail servers (MX records). `smtp_send` abstracts these complexities, providing a streamlined interface to send signed emails directly to recipient mail servers. It also handles delivery failures by automatically generating and sending rejection reports to the sender.

## Features

- **Automatic DKIM Signing**: Signs emails using RSA-SHA256 to ensure authenticity and integrity.
- **Smart MX Lookup**: Automatically resolves MX records for recipient domains using DNS-over-HTTPS (DoH).
- **Recipient Grouping**: Efficiently groups recipients by domain to minimize connections.
- **Failover Support**: Tries multiple MX servers if the primary one fails.
- **Automatic Rejection Reports**: Sends a detailed rejection email with error logs and original message attachments back to the sender if delivery fails. Uses RFC 5321 compliant null sender to prevent loops.
- **Security Best Practices**: Implements RFC 6376 recommendations.

## Usage

Here is a basic example of how to use `smtp_send` to send an email.

```rust
use smtp_send::Send;
use mail_struct::Mail;
use std::collections::{HashMap, HashSet};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Load DKIM private key from file
    let sk_bytes = std::fs::read("path/to/private.key")?; 

    // 2. Create sender using the new() method
    let sender = Send::new("default", &sk_bytes); // "default" is your DKIM selector

    // 3. Construct the Email
    // Note: In a real application, you might use a helper to construct this
    let mut host_user_li = HashMap::new();
    let mut users = HashSet::new();
    users.insert("recipient".to_string());
    host_user_li.insert("example.com".to_string(), users);

    let mail = Mail {
        sender_user: "sender".to_string(),
        sender_host: "yourdomain.com".to_string(),
        host_user_li,
        body: b"Subject: Test Email\r\n\r\nThis is a test email.".to_vec(),
    };

    // 4. Send the Email
    // Returns a SendResult struct containing success count and error details
    let result = sender.send(&mail).await;

    println!("Successfully sent to {} recipients", result.success);

    for error in result.error_li {
        println!("Error: {:?}", error);
    }

    Ok(())
}
```

> Note: See `tests/main.rs` for more comprehensive examples.

## Design

The library follows a logical flow to ensure reliable delivery:

1.  **Input**: Takes a `Mail` object and DKIM configuration (`Send` struct).
2.  **Grouping**: Recipients are grouped by their domain.
3.  **MX Resolution**: For each domain, the library queries DNS (via DoH) to find the Mail Exchange (MX) records.
4.  **Signing**: The email is cryptographically signed using the provided Private Key and Selector.
5.  **Transmission**: The library connects to the target SMTP server (port 25) and delivers the signed message.
6.  **Rejection Handling**: If delivery fails, a rejection email is automatically constructed (preserving original headers and body) and sent back to the sender with error details, using a null sender (`MAIL FROM:<>`) to prevent infinite loops.
7.  **Result**: Returns a summary of success counts and specific errors.

## Technology Stack

-   **Rust**: The core language, chosen for safety and performance.
-   **mail_send**: Handles the low-level SMTP protocol interactions.
-   **idoh**: Performs DNS-over-HTTPS lookups for MX records.
-   **sk_dkim**: Manages DKIM secret keys and signing operations.
-   **mail-parser**: Parses original emails for rejection report generation.

## Directory Structure

```
.
├── Cargo.toml      # Project configuration and dependencies
├── readme/         # Documentation files
│   ├── en.md       # English README
│   └── zh.md       # Chinese README
├── src/            # Source code
│   ├── dkim.rs     # DKIM signer implementation and caching
│   ├── error.rs    # Error definitions
│   ├── reject/     # Rejection email generation logic
│   ├── send.rs     # SMTP sending logic
│   ├── smtp.rs     # SMTP connection and sending wrapper
│   └── lib.rs      # Main library entry point
└── tests/          # Integration tests
    └── main.rs     # Usage examples and verification tests
```

## API Reference

### `struct Send`

The main configuration struct for sending emails.

-   `selector: String`: The DKIM selector.
-   `sk: Sk`: The Secret Key used for signing.

### `impl Send`

#### `fn new(selector: impl Into<String>, sk: impl AsRef<[u8]>) -> Self`

Creates a new `Send` instance.

-   **Parameters**:
    -   `selector`: The DKIM selector.
    -   `sk`: The DKIM private key bytes.
-   **Returns**:
    -   `Send`: The configured sender instance.

#### `async fn send(&self, mail: &Mail) -> SendResult`

Sends the provided email to all recipients. Automatically handles rejections for failed deliveries.

-   **Parameters**:
    -   `mail`: A reference to the `Mail` struct.
-   **Returns**:
    -   `SendResult`: A struct containing the results.

### `struct SendResult`

The result of a send operation.

-   `error_li: Vec<Error>`: A list of errors encountered during sending.
-   `success: usize`: The number of recipients the email was successfully sent to.

### `enum Error`

-   `DnsResolveFailed(String, idoh::Error)`: DNS resolution failed for a host.
-   `MxIsEmpty(String)`: No MX records found for a host.
-   `Reject(String, smtp_proto::Response<String>)`: Message rejected by the server (for a specific recipient).
-   `SendErr(String, mail_send::Error)`: Failed to send to a recipient.
-   `SmtpAllFailed(String, mail_send::Error)`: Failed to connect to or send via all available MX servers for a domain.

## History

### The Merger that Created DKIM

In the early 2000s, as email spam and phishing became rampant, two major tech giants independently worked on solutions. Yahoo! developed **DomainKeys**, focusing on verifying the DNS domain of a sender. Simultaneously, Cisco created **Identified Internet Mail (IIM)**, which proposed a signature-based authentication standard.

Recognizing that a unified standard would be more effective, these two distinct approaches were merged in 2004. This collaboration birthed **DKIM (DomainKeys Identified Mail)**. It combined the cryptographic integrity of IIM with the domain verification of DomainKeys. This unified specification eventually became an Internet Standard (RFC 6376) in 2011, becoming a cornerstone of modern email security alongside SPF and DMARC.

---

## About

This project is an open-source component of [js0.site ⋅ Refactoring the Internet Plan](https://js0.site).

We are redefining the development paradigm of the Internet in a componentized way. Welcome to follow us:

* [Google Group](https://groups.google.com/g/js0-site)
* [js0site.bsky.social](https://bsky.app/profile/js0site.bsky.social)

---

<a id="zh"></a>

# smtp_send : 支持 DKIM 签名的安全 SMTP 邮件发送工具

`smtp_send` 是一个强大的 Rust 库，专为通过 SMTP 发送邮件而设计，内置了 DKIM 签名支持、自动 MX 记录查询以及自动拒信处理功能。它简化了发送认证邮件的流程，确保了高送达率和安全性。

## 目录

- [简介](#简介)
- [功能特性](#功能特性)
- [使用演示](#使用演示)
- [设计思路](#设计思路)
- [技术堆栈](#技术堆栈)
- [目录结构](#目录结构)
- [API 参考](#api-reference)
- [历史故事](#历史故事)

## 简介

通过程序发送邮件通常涉及复杂的配置，尤其是涉及到 DKIM（域名密钥识别邮件）等认证标准以及查找正确的邮件服务器（MX 记录）时。`smtp_send` 抽象了这些复杂性，提供了一个精简的接口，可以直接将带有签名的邮件发送到收件人的邮件服务器。它还能处理投递失败的情况，自动生成并发送拒信报告给发件人。

## 功能特性

-   **自动 DKIM 签名**：使用 RSA-SHA256 对邮件进行签名，确保真实性和完整性。
-   **智能 MX 查询**：使用 DNS-over-HTTPS (DoH) 自动解析收件人域名的 MX 记录。
-   **收件人分组**：按域名高效分组收件人，以最小化连接数。
-   **故障转移支持**：如果主 MX 服务器失败，自动尝试其他 MX 服务器。
-   **自动拒信报告**：如果投递失败，自动向发件人发送包含错误日志和原始邮件附件的详细拒信。使用符合 RFC 5321 标准的空发件人以防止邮件循环。
-   **安全最佳实践**：实现了 RFC 6376 建议。

## 使用演示

以下是如何使用 `smtp_send` 发送邮件的基本示例。

```rust
use smtp_send::Send;
use mail_struct::Mail;
use std::collections::{HashMap, HashSet};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 从文件加载 DKIM 私钥
    let sk_bytes = std::fs::read("path/to/private.key")?; 

    // 2. 使用 new() 方法创建发送器
    let sender = Send::new("default", &sk_bytes); // "default" 是你的 DKIM 选择器

    // 3. 构建邮件
    // 注意：在实际应用中，你可能会使用辅助函数来构建它
    let mut host_user_li = HashMap::new();
    let mut users = HashSet::new();
    users.insert("recipient".to_string());
    host_user_li.insert("example.com".to_string(), users);

    let mail = Mail {
        sender_user: "sender".to_string(),
        sender_host: "yourdomain.com".to_string(),
        host_user_li,
        body: b"Subject: Test Email\r\n\r\nThis is a test email.".to_vec(),
    };

    // 4. 发送邮件
    // 返回 SendResult 结构体，包含成功数量和错误详情
    let result = sender.send(&mail).await;

    println!("成功发送给 {} 位收件人", result.success);

    for error in result.error_li {
        println!("错误: {:?}", error);
    }

    Ok(())
}
```

> 注意：更多详细示例请参考 `tests/main.rs`。

## 设计思路

本库遵循逻辑流程以确保可靠的交付：

1.  **输入**：接收 `Mail` 对象和 DKIM 配置（`Send` 结构体）。
2.  **分组**：收件人按其域名分组。
3.  **MX 解析**：对于每个域名，库通过 DNS (DoH) 查询邮件交换 (MX) 记录。
4.  **签名**：使用提供的私钥和选择器对邮件进行加密签名。
5.  **传输**：库连接到目标 SMTP 服务器（端口 25）并投递签名消息。
6.  **拒信处理**：如果投递失败，自动构建拒信（保留原始头部和正文）并将错误详情发送回发件人，使用空发件人 (`MAIL FROM:<>`) 以防止无限循环。
7.  **结果**：返回成功计数和具体错误的摘要。

## 技术堆栈

-   **Rust**：核心语言，因其安全性和性能而选择。
-   **mail_send**：处理底层 SMTP 协议交互。
-   **idoh**：执行 DNS-over-HTTPS 查询以获取 MX 记录。
-   **sk_dkim**：管理 DKIM 私钥和签名操作。
-   **mail-parser**：解析原始邮件以生成拒信报告。

## 目录结构

```
.
├── Cargo.toml      # 项目配置和依赖
├── readme/         # 文档文件
│   ├── en.md       # 英文 README
│   └── zh.md       # 中文 README
├── src/            # 源代码
│   ├── dkim.rs     # DKIM 签名器实现和缓存
│   ├── error.rs    # 错误定义
│   ├── reject/     # 拒信生成逻辑
│   ├── send.rs     # SMTP 发送逻辑
│   ├── smtp.rs     # SMTP 连接和发送封装
│   └── lib.rs      # 主库入口点
└── tests/          # 集成测试
    └── main.rs     # 使用示例和验证测试
```

## API 参考

### `struct Send`

用于发送邮件的主要配置结构体。

-   `selector: String`: DKIM 选择器。
-   `sk: Sk`: 用于签名的私钥。

### `impl Send`

#### `fn new(selector: impl Into<String>, sk: impl AsRef<[u8]>) -> Self`

创建一个新的 `Send` 实例。

-   **参数**:
    -   `selector`: DKIM 选择器。
    -   `sk`: DKIM 私钥字节。
-   **返回**:
    -   `Send`: 配置好的发送器实例。

#### `async fn send(&self, mail: &Mail) -> SendResult`

将提供的邮件发送给所有收件人。自动处理投递失败的拒信。

-   **参数**:
    -   `mail`: `Mail` 结构体的引用。
-   **返回**:
    -   `SendResult`: 包含结果的结构体。

### `struct SendResult`

发送操作的结果。

-   `error_li: Vec<Error>`: 发送过程中遇到的错误列表。
-   `success: usize`: 邮件成功发送到的收件人数量。

### `enum Error`

-   `DnsResolveFailed(String, idoh::Error)`: 主机 DNS 解析失败。
-   `MxIsEmpty(String)`: 未找到主机的 MX 记录。
-   `Reject(String, smtp_proto::Response<String>)`: 邮件被服务器拒绝（针对特定收件人）。
-   `SendErr(String, mail_send::Error)`: 发送到特定收件人失败。
-   `SmtpAllFailed(String, mail_send::Error)`: 对域名的所有 MX 服务器连接或发送均失败。

## 历史故事

### DKIM 诞生的融合

在 2000 年代初，随着垃圾邮件和网络钓鱼变得猖獗，两大科技巨头分别致力于解决方案。Yahoo! 开发了 **DomainKeys**，专注于验证发件人的 DNS 域名。与此同时，Cisco 创建了 **Identified Internet Mail (IIM)**，提出了一种基于签名的认证标准。

认识到统一的标准将更有效，这两种不同的方法在 2004 年合并。这次合作诞生了 **DKIM (DomainKeys Identified Mail)**。它结合了 IIM 的加密完整性和 DomainKeys 的域验证。这一统一规范最终在 2011 年成为互联网标准 (RFC 6376)，与 SPF 和 DMARC 一起成为现代电子邮件安全的基石。

---

## 关于

本项目为 [js0.site ⋅ 重构互联网计划](https://js0.site) 的开源组件。

我们正在以组件化的方式重新定义互联网的开发范式，欢迎关注：

* [谷歌邮件列表](https://groups.google.com/g/js0-site)
* [js0site.bsky.social](https://bsky.app/profile/js0site.bsky.social)
