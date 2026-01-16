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

- **Core Structure**: Defines `Mail`, `UserMail`, and `HostUserLi` structs to represent email data with type-safe domain grouping.
- **Serialization**: Optional `encode` and `decode` features using `bitcode` for high-performance binary serialization.
- **SMTP Integration**: Optional `send` feature with domain-based recipient grouping for efficient email delivery.
- **Type Safety**: Leverages Rust's type system with custom types like `HostUserLi` to ensure data integrity and prevent misuse.

## Usage

Add `mail_struct` to your `Cargo.toml`:

```toml
[dependencies]
mail_struct = { version = "0.1.18", features = ["send", "encode", "decode"] }
```

### Creating and Grouping Emails by Domain

```rust
use mail_struct::Mail;

#[cfg(feature = "send")]
async fn example() {
    // Mail::new() automatically:
    // - Normalizes and validates email addresses using xmail::norm_user_host
    // - Filters out invalid email addresses
    // - Deduplicates recipients
    // Returns Option<Mail>, None if no valid recipients
    let mail = Mail::new(
        "sender@example.com",
        vec![
            "user1@gmail.com",
            "user2@yahoo.com",
            "user3@gmail.com",
            "user1@gmail.com",  // Duplicate - will be removed
            "invalid-email",     // Invalid - will be filtered out
        ],
        b"Hello, this is a test email!",
    ).unwrap(); // Handle None in production

    // Directly iterate over &mail to get MailMessage grouped by domain
    for msg in &mail {
        println!("Sending to domain: {}", msg.domain);
        
        // 1. Try batch sending to all recipients in this domain
        // MailMessage implements IntoMessage trait, can be passed directly to client.send()
        if let Err(e) = client.send(&msg).await {
            println!("✗ Batch send failed: {}, sending individually", e);
            
            // 2. If batch send fails, use IntoIterator to send individually
            // This uses a custom zero-allocation iterator
            for individual_message in msg {
                match client.send(individual_message).await {
                    Ok(_) => println!("  ✓ Individual send successful"),
                    Err(e) => println!("  ✗ Individual send failed: {}", e),
                }
            }
        } else {
            println!("✓ Batch send successful");
        }
    }
}
```

## Design Philosophy

The library follows a separation of concerns principle. The core `lib.rs` defines the data structures (`Mail`, `UserMail`), keeping the base dependency footprint minimal. Functionalities like serialization and sending are gated behind feature flags (`encode`, `decode`, `send`), allowing users to opt-in only for what they need.

When the `send` feature is active, the `send.rs` module implements `IntoIterator` for `&Mail`, returning `MailMessage` instances grouped by domain. This optimization reduces the number of SMTP connections needed and improves delivery efficiency. Additionally, `MailMessage` also implements `IntoIterator`, providing a zero-overhead fallback mechanism that ensures individual delivery to valid recipients when batch sending fails due to invalid recipient addresses.

## Tech Stack

- **Rust**: Core language.
- **xmail**: Email validation and normalization.
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
│   ├── host_user_li.rs # HostUserLi type for domain-based recipient grouping
│   └── send.rs         # SMTP message logic (feature: send)
└── tests
    └── main.rs         # Integration tests
```

## API Documentation

### `struct Mail`

Represents a basic email message.

- `sender_user: String`: The user part of the sender's email address.
- `sender_host: String`: The domain part of the sender's email address.
- `host_user_li: HostUserLi`: Recipients grouped by domain using a type-safe wrapper around `HashMap<String, HashSet<String>>`.
- `body: Vec<u8>`: The raw body content of the email.

### `struct HostUserLi`

A type-safe wrapper for domain-based recipient grouping that implements `Deref<Target = HashMap<String, HashSet<String>>>`.

#### Methods

##### `add(&mut self, mail: impl AsRef<str>) -> bool`
Adds an email address with automatic validation and normalization using `xmail::norm_user_host`. Returns `true` if the email was valid and added successfully, `false` otherwise. Invalid emails are logged and ignored.

##### `user_li(&self, host: &str) -> Option<&HashSet<String>>`
Returns the set of users for a given host domain, or `None` if the domain doesn't exist.

#### Trait Implementations

##### `Deref<Target = HashMap<String, HashSet<String>>>`
Allows direct read-only access to underlying HashMap methods like `len()`, `is_empty()`, `iter()`, `get()`, etc.

##### `DerefMut<Target = HashMap<String, HashSet<String>>>`
Provides mutable access to the underlying HashMap, enabling direct manipulation using methods like `entry()`, `insert()`, `remove()`, etc. This allows for advanced operations while maintaining type safety.

##### `FromIterator<T: AsRef<str>>`
Enables creation from any iterator of string-like items using `from_iter()`. Each item is validated and added automatically, making it easy to create `HostUserLi` from collections of email addresses.

#### Methods

##### `new(sender: impl AsRef<str>, to_li: impl IntoIterator<Item = impl AsRef<str>>, body: impl Into<Vec<u8>>) -> Option<Self>`

Creates a new `Mail` instance with automatic email processing:
- **Sender Validation**: Uses `xmail::norm_user_host` to validate and normalize the sender address.
- **Recipient Processing**: Uses `HostUserLi::from_iter()` to process all recipients in a functional style.
- **Automatic Validation**: Each recipient is validated and normalized using `xmail::norm_user_host`.
- **Filtering**: Invalid email addresses are automatically filtered out and logged.
- **Deduplication**: Recipients are automatically deduplicated using `HashSet` within each domain.
- **Grouping**: Recipients are organized by domain (host) for efficient processing.
- **Return**: Returns `None` if no valid recipients remain after filtering, or if the sender is invalid.

#### Trait Implementations

##### `IntoIterator` for `&'a Mail` (requires `send` feature)

Implements `IntoIterator` for `&Mail`, returning a custom `MailIter<'a>` iterator.
- **Domain Grouping**: Automatically groups recipients by email domain, with each `MailMessage` containing all recipients for the same domain.
- **Lazy Construction**: The iterator lazily creates `MailMessage` instances during iteration, avoiding pre-allocation of a `Vec`.
- **Efficient Delivery**: Multiple recipients in the same domain can be delivered in a single SMTP transaction.

### `struct UserMail`

A wrapper around `Mail` associating it with a user ID.

- `mail: Mail`: The email content.
- `user_id: u64`: The unique identifier of the user associated with this mail.

### `struct MailMessage<'a>` (requires `send` feature)

Represents an email grouped by recipient domain.

- `sender_user: &'a str`: The user part of the sender's email address.
- `sender_host: &'a str`: The domain part of the sender's email address.
- `domain: &'a str`: The recipient domain name (e.g., "gmail.com").
- `to_li: Vec<Address<'a>>`: List of all recipient addresses for this domain.
- `body: &'a [u8]`: The email body content.

#### Trait Implementations

##### `IntoMessage<'a>`

Implements the `mail_send::smtp::message::IntoMessage` trait, allowing `MailMessage` to be directly converted into an SMTP message containing all recipients. This enables `MailMessage` to be passed directly to `client.send()` for batch sending.

Also implements `IntoMessage` for `&MailMessage`, allowing message creation without consuming ownership.

##### `IntoIterator`

Implements the `IntoIterator` trait, returning a custom `MailMessageIter` iterator.
- **Zero Allocation**: The iterator lazily constructs `Message` instances during iteration, avoiding the overhead of creating an intermediate `Vec`.
- **Fallback Strategy**: Converts `MailMessage` into multiple individual `Message` instances (each containing one recipient). This is useful when batch sending fails, allowing for individual delivery to ensure valid recipients receive the email.

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

- **核心结构**: 定义了 `Mail`、`UserMail` 和 `HostUserLi` 结构体来表示邮件数据，提供类型安全的域名分组。
- **序列化**: 通过 `encode` 和 `decode` 特性支持使用 `bitcode` 进行高性能的二进制序列化。
- **SMTP 集成**: 可选的 `send` 特性支持按域名分组收件人，实现高效的邮件投递。
- **类型安全**: 利用 Rust 的类型系统和自定义类型如 `HostUserLi` 确保数据完整性并防止误用。

## 使用指南

在 `Cargo.toml` 中添加 `mail_struct`：

```toml
[dependencies]
mail_struct = { version = "0.1.18", features = ["send", "encode", "decode"] }
```

### 创建并按域名分组邮件

```rust
use mail_struct::Mail;

#[cfg(feature = "send")]
async fn example() {
    // Mail::new() 会自动：
    // - 使用 xmail::norm_user_host 规范化和验证邮箱地址
    // - 过滤掉无效的邮箱地址
    // - 对收件人进行去重
    // 返回 Option<Mail>，如果没有有效收件人则返回 None
    let mail = Mail::new(
        "sender@example.com",
        vec![
            "user1@gmail.com",
            "user2@yahoo.com",
            "user3@gmail.com",
            "user1@gmail.com",  // 重复 - 会被去除
            "invalid-email",     // 无效 - 会被过滤
        ],
        b"Hello, this is a test email!",
    ).unwrap(); // 生产环境中请处理 None

    // 直接迭代 &mail 即可按域名分组获取 MailMessage
    for msg in &mail {
        println!("Sending to domain: {}", msg.domain);
        
        // 1. 尝试批量发送给该域名的所有收件人
        // MailMessage 实现了 IntoMessage trait，可以直接传给 client.send()
        if let Err(e) = client.send(&msg).await {
            println!("✗ 批量发送失败: {}，开始逐个投递", e);
            
            // 2. 如果批量发送失败，利用 IntoIterator 逐个发送
            // 使用自定义迭代器，零内存分配
            for individual_message in msg {
                match client.send(individual_message).await {
                    Ok(_) => println!("  ✓ 单个邮件发送成功"),
                    Err(e) => println!("  ✗ 单个邮件发送失败: {}", e),
                }
            }
        } else {
             println!("✓ 批量发送成功");
        }
    }
}
```

## 设计理念

本库遵循关注点分离原则。核心 `lib.rs` 定义了数据结构（`Mail`, `UserMail`），保持基础依赖最小化。序列化和发送等功能通过特性标志（`encode`, `decode`, `send`）进行门控，允许用户仅按需开启。

当 `send` 特性激活时，`send.rs` 模块为 `&Mail` 实现了 `IntoIterator`，返回按域名分组的 `MailMessage`。这种优化减少了所需的 SMTP 连接数量并提高了投递效率。同时，`MailMessage` 也实现了 `IntoIterator`，提供了零开销的降级方案，确保在部分收件人地址错误导致批量发送失败时，仍能逐个投递给其他有效的收件人。

## 技术栈

- **Rust**: 核心开发语言。
- **xmail**: 邮箱验证和规范化。
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
│   ├── host_user_li.rs # HostUserLi 类型，用于基于域名的收件人分组
│   └── send.rs         # SMTP 消息逻辑 (特性: send)
└── tests
    └── main.rs         # 集成测试
```

## API 文档

### `struct Mail`

表示一个基础的邮件消息。

- `sender_user: String`: 发件人邮箱地址的用户部分。
- `sender_host: String`: 发件人邮箱地址的域名部分。
- `host_user_li: HostUserLi`: 使用类型安全包装器对 `HashMap<String, HashSet<String>>` 进行域名分组的收件人。
- `body: Vec<u8>`: 邮件的原始正文内容。

### `struct HostUserLi`

基于域名的收件人分组的类型安全包装器，实现了 `Deref<Target = HashMap<String, HashSet<String>>>`。

#### 方法

##### `add(&mut self, mail: impl AsRef<str>) -> bool`
添加邮箱地址并自动使用 `xmail::norm_user_host` 进行验证和规范化。如果邮箱有效并成功添加则返回 `true`，否则返回 `false`。无效邮箱会被记录日志并忽略。

##### `user_li(&self, host: &str) -> Option<&HashSet<String>>`
返回给定主机域名的用户集合，如果域名不存在则返回 `None`。

#### Trait 实现

##### `Deref<Target = HashMap<String, HashSet<String>>>`
允许直接只读访问底层 HashMap 的方法，如 `len()`、`is_empty()`、`iter()`、`get()` 等。

##### `DerefMut<Target = HashMap<String, HashSet<String>>>`
提供对底层 HashMap 的可变访问，支持使用 `entry()`、`insert()`、`remove()` 等方法进行直接操作。这允许进行高级操作的同时保持类型安全。

##### `FromIterator<T: AsRef<str>>`
支持使用 `from_iter()` 从任何字符串类型的迭代器创建实例。每个项目都会自动验证和添加，便于从邮箱地址集合创建 `HostUserLi`。

#### 方法

##### `new(sender: impl AsRef<str>, to_li: impl IntoIterator<Item = impl AsRef<str>>, body: impl Into<Vec<u8>>) -> Option<Self>`

创建新的 `Mail` 实例，自动处理邮箱：
- **发件人验证**：使用 `xmail::norm_user_host` 验证和规范化发件人地址。
- **收件人处理**：使用 `HostUserLi::from_iter()` 以函数式风格处理所有收件人。
- **自动验证**：每个收件人都使用 `xmail::norm_user_host` 进行验证和规范化。
- **过滤**：无效的邮箱地址会自动过滤并记录日志。
- **去重**：在每个域名内使用 `HashSet` 自动去除重复的收件人。
- **分组**：按域名（主机）组织收件人，提高处理效率。
- **返回**：如果过滤后没有有效收件人或发件人无效，则返回 `None`。

#### Trait 实现

##### `IntoIterator` for `&'a Mail` (需要 `send` 特性)

为 `&Mail` 实现了 `IntoIterator`，返回自定义的 `MailIter<'a>` 迭代器。
- **按域名分组**：自动按邮件域名分组收件人，每个 `MailMessage` 包含同一域名的所有收件人。
- **惰性构造**：迭代器在遍历时惰性创建 `MailMessage` 实例，避免预先分配 `Vec`。
- **高效投递**：同一域名的多个收件人可以在单个 SMTP 事务中投递。

### `struct UserMail`

`Mail` 的包装器，将其与用户 ID 关联。

- `mail: Mail`: 邮件内容。
- `user_id: u64`: 与此邮件关联的用户的唯一标识符。

### `struct MailMessage<'a>` (需要 `send` 特性)

表示按收件人域名分组的邮件。

- `sender_user: &'a str`: 发件人邮箱地址的用户部分。
- `sender_host: &'a str`: 发件人邮箱地址的域名部分。
- `domain: &'a str`: 收件人域名（例如 "gmail.com"）。
- `to_li: Vec<Address<'a>>`: 该域名下的所有收件人地址列表。
- `body: &'a [u8]`: 邮件正文内容。

#### Trait 实现

##### `IntoMessage<'a>`

实现了 `mail_send::smtp::message::IntoMessage` trait，允许 `MailMessage` 直接转换为包含所有收件人的 SMTP 消息。这使得 `MailMessage` 可以直接传递给 `client.send()` 进行批量发送。

同时为 `&MailMessage` 实现了 `IntoMessage`，允许在不消耗所有权的情况下创建消息。

##### `IntoIterator`

实现了 `IntoIterator` trait，返回自定义的 `MailMessageIter` 迭代器。
- **零分配**：迭代器在遍历时惰性构造 `Message`，避免了创建中间 `Vec` 的开销。
- **降级策略**：将 `MailMessage` 转换为多个独立的 `Message`（每个包含一个收件人）。这在批量发送失败时非常有用，可以逐个投递以确保有效地址的收件人能收到邮件。

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
