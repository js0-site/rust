[English](#en) | [中文](#zh)

---

<a id="en"></a>

# mail_struct : Minimalist Email Structure for Rust

`mail_struct` is a lightweight Rust library designed to define a clear and efficient structure for email messages. It provides optional integration with `bitcode` for efficient encoding/decoding and `mail-send` for SMTP transmission with domain-based grouping, making it a versatile choice for email handling in Rust applications.

## Table of Contents

- [Features](#features)
- [Usage](#usage)
- [Design Philosophy](#design-philosophy)
- [Tech Stack](#tech-stack)
- [Directory Structure](#directory-structure)
- [API Documentation](#api-documentation)
- [Historical Context](#historical-context)

## Features

- **Core Structure**: Defines `Mail` and `UserMail` structs to represent email data.
- **Serialization**: Optional `encode` and `decode` features using `bitcode` for high-performance binary serialization.
- **SMTP Integration**: Optional `send` feature with domain-based recipient grouping for efficient email delivery.
- **Type Safety**: Leverages Rust's type system to ensure data integrity.

## Usage

Add `mail_struct` to your `Cargo.toml`:

```toml
[dependencies]
mail_struct = { version = "0.1.7", features = ["send", "encode", "decode"] }
```

### Creating and Grouping Emails by Domain

```rust
use mail_struct::Mail;

#[cfg(feature = "send")]
async fn example() {
    let mail = Mail {
        sender: "sender@example.com".to_string(),
        to_li: vec![
            "user1@gmail.com".to_string(),
            "user2@yahoo.com".to_string(),
            "user3@gmail.com".to_string(),
        ],
        body: b"Hello, this is a test email!".to_vec(),
    };

    // With the 'send' feature enabled, group recipients by domain
    let domain_mail = mail.domain_mail();
    
    for item in domain_mail {
        println!("Sending to domain: {}", item.domain);
        // item.mail is a mail_send::smtp::message::Message
        // client.send(item.mail).await?;
    }
}
```

## Design Philosophy

The library follows a separation of concerns principle. The core `lib.rs` defines the data structures (`Mail`, `UserMail`), keeping the base dependency footprint minimal. Functionalities like serialization and sending are gated behind feature flags (`encode`, `decode`, `send`), allowing users to opt-in only for what they need.

When the `send` feature is active, the `send.rs` module provides the `domain_mail` method that groups recipients by their email domain. This optimization reduces the number of SMTP connections needed and improves delivery efficiency, especially when sending to multiple recipients across different domains.

## Tech Stack

- **Rust**: Core language.
- **bitcode** (Optional): For fast binary encoding and decoding.
- **mail-send** (Optional): For SMTP message construction and sending.

## Directory Structure

```
.
├── Cargo.toml          # Project configuration
├── README.md           # Main documentation
├── readme              # Documentation in specific languages
│   ├── en.md           # English README
│   └── zh.md           # Chinese README
├── src
│   ├── lib.rs          # Core struct definitions and feature gates
│   └── send.rs         # Domain grouping and SMTP message logic (feature: send)
└── tests
    └── main.rs         # Integration tests
```

## API Documentation

### `struct Mail`

Represents a basic email message.

- `sender: String`: The email address of the sender.
- `to_li: Vec<String>`: A list of recipient email addresses.
- `body: Vec<u8>`: The raw body content of the email.

#### Methods

##### `domain_mail<'a>(&'a self) -> Vec<DomainMail<'a>>` (requires `send` feature)

Groups recipients by their email domain and creates a `DomainMail` for each domain. This allows for efficient batch sending where multiple recipients in the same domain can be delivered in a single SMTP transaction.

### `struct UserMail`

A wrapper around `Mail` associating it with a user ID.

- `mail: Mail`: The email content.
- `user_id: u64`: The unique identifier of the user associated with this mail.

### `struct DomainMail<'a>` (requires `send` feature)

Represents an email grouped by recipient domain.

- `domain: &'a str`: The domain name (e.g., "gmail.com").
- `mail: Message<'a>`: The ready-to-send SMTP message containing all recipients for this domain.

## Historical Context

**RFC 822 and the Separation of Envelope and Content**

The design of email systems dates back to the early 1980s with the publication of **RFC 822** (Standard for the Format of ARPA Internet Text Messages) and **RFC 821** (Simple Mail Transfer Protocol). A key architectural decision was the separation of the "envelope" (handled by SMTP for routing) from the "content" (the message headers and body defined by RFC 822).

`mail_struct` honors this tradition by focusing on the *structure* of the message (the content), while delegating the *transport* (the envelope and transmission) to specialized libraries like `mail-send`. This modular approach mirrors the original design philosophy of the internet's most enduring communication protocol, ensuring flexibility and maintainability.

---

## About

This project is an open-source component of [js0.site ⋅ Refactoring the Internet Plan](https://js0.site).

We are redefining the development paradigm of the Internet in a componentized way. Welcome to follow us:

* [Google Group](https://groups.google.com/g/js0-site)
* [js0site.bsky.social](https://bsky.app/profile/js0site.bsky.social)

---

<a id="zh"></a>

# mail_struct : 极简 Rust 邮件结构库

`mail_struct` 是一个轻量级的 Rust 库，旨在为邮件消息定义清晰且高效的结构。它提供了与 `bitcode`（用于高效编码/解码）和 `mail-send`（用于基于域名分组的 SMTP 传输）的可选集成，使其成为 Rust 应用中处理邮件的灵活选择。

## 目录

- [功能特性](#功能特性)
- [使用指南](#使用指南)
- [设计理念](#设计理念)
- [技术栈](#技术栈)
- [目录结构](#目录结构)
- [API 文档](#api-文档)
- [历史背景](#历史背景)

## 功能特性

- **核心结构**: 定义了 `Mail` 和 `UserMail` 结构体来表示邮件数据。
- **序列化**: 通过 `encode` 和 `decode` 特性支持使用 `bitcode` 进行高性能的二进制序列化。
- **SMTP 集成**: 可选的 `send` 特性支持按域名分组收件人，实现高效的邮件投递。
- **类型安全**: 利用 Rust 的类型系统确保数据完整性。

## 使用指南

在 `Cargo.toml` 中添加 `mail_struct`：

```toml
[dependencies]
mail_struct = { version = "0.1.7", features = ["send", "encode", "decode"] }
```

### 创建并按域名分组邮件

```rust
use mail_struct::Mail;

#[cfg(feature = "send")]
async fn example() {
    let mail = Mail {
        sender: "sender@example.com".to_string(),
        to_li: vec![
            "user1@gmail.com".to_string(),
            "user2@yahoo.com".to_string(),
            "user3@gmail.com".to_string(),
        ],
        body: b"Hello, this is a test email!".to_vec(),
    };

    // 启用 'send' 特性后，可以按域名分组收件人
    let domain_mail = mail.domain_mail();
    
    for item in domain_mail {
        println!("Sending to domain: {}", item.domain);
        // item.mail 是 mail_send::smtp::message::Message
        // client.send(item.mail).await?;
    }
}
```

## 设计理念

本库遵循关注点分离原则。核心 `lib.rs` 定义了数据结构（`Mail`, `UserMail`），保持基础依赖最小化。序列化和发送等功能通过特性标志（`encode`, `decode`, `send`）进行门控，允许用户仅按需开启。

当 `send` 特性激活时，`send.rs` 模块提供了 `domain_mail` 方法，该方法按邮件域名分组收件人。这种优化减少了所需的 SMTP 连接数量并提高了投递效率，特别是在向不同域名的多个收件人发送邮件时。

## 技术栈

- **Rust**: 核心开发语言。
- **bitcode** (可选): 用于快速二进制编码和解码。
- **mail-send** (可选): 用于构建和发送 SMTP 消息。

## 目录结构

```
.
├── Cargo.toml          # 项目配置
├── README.md           # 主文档
├── readme              # 多语言文档
│   ├── en.md           # 英文 README
│   └── zh.md           # 中文 README
├── src
│   ├── lib.rs          # 核心结构定义及特性门控
│   └── send.rs         # 域名分组及 SMTP 消息逻辑 (特性: send)
└── tests
    └── main.rs         # 集成测试
```

## API 文档

### `struct Mail`

表示一个基础的邮件消息。

- `sender: String`: 发件人邮箱地址。
- `to_li: Vec<String>`: 收件人邮箱地址列表。
- `body: Vec<u8>`: 邮件的原始正文内容。

#### 方法

##### `domain_mail<'a>(&'a self) -> Vec<DomainMail<'a>>` (需要 `send` 特性)

按邮件域名分组收件人，并为每个域名创建一个 `DomainMail`。这允许高效的批量发送，同一域名的多个收件人可以在单个 SMTP 事务中投递。

### `struct UserMail`

`Mail` 的包装器，将其与用户 ID 关联。

- `mail: Mail`: 邮件内容。
- `user_id: u64`: 与此邮件关联的用户的唯一标识符。

### `struct DomainMail<'a>` (需要 `send` 特性)

表示按收件人域名分组的邮件。

- `domain: &'a str`: 域名（例如 "gmail.com"）。
- `mail: Message<'a>`: 准备发送的 SMTP 消息，包含该域名的所有收件人。

## 历史背景

**RFC 822 与信封/内容的分离**

电子邮件系统的设计可以追溯到 20 世纪 80 年代初 **RFC 822**（ARPA 互联网文本消息格式标准）和 **RFC 821**（简单邮件传输协议）的发布。一个关键的架构决策是将"信封"（由 SMTP 处理用于路由）与"内容"（由 RFC 822 定义的消息头和正文）分离开来。

`mail_struct` 秉承了这一传统，专注于消息的**结构**（内容），而将**传输**（信封和发送）委托给像 `mail-send` 这样的专用库。这种模块化的方法反映了互联网最持久通信协议的原始设计哲学，确保了灵活性和可维护性。

---

## 关于

本项目为 [js0.site ⋅ 重构互联网计划](https://js0.site) 的开源组件。

我们正在以组件化的方式重新定义互联网的开发范式，欢迎关注：

* [谷歌邮件列表](https://groups.google.com/g/js0-site)
* [js0site.bsky.social](https://bsky.app/profile/js0site.bsky.social)
