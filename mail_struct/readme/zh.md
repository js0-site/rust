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
