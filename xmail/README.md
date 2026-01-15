[English](#en) | [中文](#zh)

---

<a id="en"></a>

# xmail : Normalize and Parse Email Addresses with Ease

[English](readme/en.md) | [中文](readme/zh.md)

## Table of Contents
- [Introduction](#introduction)
- [Features](#features)
- [Usage](#usage)
- [Design](#design)
- [Tech Stack](#tech-stack)
- [Directory Structure](#directory-structure)
- [API Reference](#api-reference)
- [History](#history)

## Introduction
`xmail` is a lightweight Rust library designed to parse and normalize email addresses. It handles common tasks such as splitting the user and host parts, converting to lowercase, trimming whitespace, and decoding Punycode for internationalized domain names (IDN). It also offers optional support for extracting the Top-Level Domain (TLD).

## Features
- **Split User and Host**: Efficiently separates the local part (user) from the domain part (host).
- **Normalization**: Automatically trims whitespace and converts the email address to lowercase.
- **Punycode Support**: Decodes Punycode-encoded domains (e.g., `xn--...`) into their Unicode representation.
- **TLD Extraction**: Optionally extracts the effective Top-Level Domain (requires `tld` feature).
- **Robust Parsing**: Handles edge cases like missing `@` symbols, empty user parts, or invalid domains (missing `.`, or starting/ending with `.`) gracefully using `Option`.

## Usage

Add `xmail` to your `Cargo.toml`:

```toml
[dependencies]
xmail = "0.1.15"
# For TLD support:
# xmail = { version = "0.1.15", features = ["tld"] }
```

### Examples

```rust
fn main() {
    // Basic normalization
    let email = "  User@Example.COM  ";
    let normalized = xmail::norm(email).unwrap();
    assert_eq!(normalized, "user@example.com");

    // Punycode decoding
    // xn--yfro4i67o is Punycode for "新加坡" (Singapore)
    let idn_email = "user@site.xn--yfro4i67o";
    let (user, host) = xmail::norm_user_host(idn_email).unwrap();
    assert_eq!(host, "site.新加坡");

    // TLD extraction (requires "tld" feature)
    #[cfg(feature = "tld")]
    {
        let (_, tld) = xmail::norm_tld("user@example.co.uk").unwrap();
        assert_eq!(tld, "co.uk");
    }
}
```

## Design
The library processes email addresses through a pipeline:
1.  **Input**: Accepts any string type that implements `AsRef<str>`.
2.  **Split**: The `user_host` function locates the first `@` symbol to separate the user and host parts.
3.  **Normalize**:
    *   Trims leading/trailing whitespace.
    *   Converts characters to lowercase.
    *   Iterates through domain parts (separated by `.`) and decodes any starting with `xn--` using Punycode.
4.  **Output**: Returns the processed parts or the reconstructed email string wrapped in `Option`.

## Tech Stack
- **Rust**: The core language for safety and performance.
- **xstr**: Used for efficient string manipulation (trimming, cutting).
- **punycode**: Handles the decoding of Internationalized Domain Names.
- **xtld**: (Optional) Provides accurate Public Suffix List based TLD extraction.

## Directory Structure
```
.
├── Cargo.toml          # Project configuration
├── README.mdt          # Template for generating READMEs
├── readme/             # Documentation storage
│   ├── en.md           # English documentation
│   └── zh.md           # Chinese documentation
├── src/
│   └── lib.rs          # Core library logic
└── tests/
    └── main.rs         # Integration tests
```

## API Reference

### `user_host(mail: impl AsRef<str>) -> Option<(String, String)>`
Splits the email into user and host parts without normalization. Returns `None` if:
- `@` is missing.
- The user part is empty.
- The host part does not contain a dot (`.`).
- The host part starts or ends with a dot.

### `norm_user_host(mail: impl AsRef<str>) -> Option<(String, String)>`
Normalizes the email (lowercase, trim) and decodes Punycode in the host part.

### `norm(mail: impl AsRef<str>) -> Option<String>`
Normalizes the email and returns it as a single string `user@host`.

### `norm_tld(mail: impl AsRef<str>) -> Option<(String, String)>`
*Requires `tld` feature.* Normalizes the email and returns a tuple containing the full email address and the extracted TLD.

## History
The `@` symbol in email addresses was introduced by Ray Tomlinson in 1971 to separate the user from the host machine. He chose it simply because it was a rarely used character on the keyboard that made sense ("user" at "host").

The term **Punycode**, used in this library for international domains, has an interesting origin. It rhymes with "Unicode" and is "puny" in three ways: it uses a small character set (ASCII), generates short strings, and has a small implementation footprint. This clever encoding allows non-ASCII characters (like Chinese or Emoji) to exist within the legacy ASCII-only DNS system.

---

## About

This project is an open-source component of [js0.site ⋅ Refactoring the Internet Plan](https://js0.site).

We are redefining the development paradigm of the Internet in a componentized way. Welcome to follow us:

* [Google Group](https://groups.google.com/g/js0-site)
* [js0site.bsky.social](https://bsky.app/profile/js0site.bsky.social)

---

<a id="zh"></a>

# xmail : 轻松实现邮件地址的标准化与解析

[English](en.md) | [中文](zh.md)

## 目录
- [简介](#简介)
- [功能特性](#功能特性)
- [使用说明](#使用说明)
- [设计思路](#设计思路)
- [技术栈](#技术栈)
- [目录结构](#目录结构)
- [API 参考](#api-参考)
- [历史趣闻](#历史趣闻)

## 简介
`xmail` 是一个轻量级的 Rust 库，专为解析和标准化电子邮件地址而设计。它能高效处理常见任务，如分离用户名和域名、转换为小写、去除首尾空格以及解码国际化域名（IDN）的 Punycode。此外，它还提供了提取顶级域名（TLD）的可选功能。

## 功能特性
- **分离用户与主机**：高效地将邮件地址拆分为本地部分（用户名）和域名部分（主机）。
- **标准化处理**：自动去除空白字符并将邮件地址转换为小写。
- **Punycode 支持**：将 Punycode 编码的域名（如 `xn--...`）解码为原本的 Unicode 字符。
- **TLD 提取**：可选提取有效的顶级域名（需要开启 `tld` 特性）。
- **健壮的解析**：使用 `Option` 优雅地处理缺失 `@` 符号、空用户名或无效域名（缺失 `.`，或以 `.` 开头/结尾）等边界情况。

## 使用说明

在 `Cargo.toml` 中添加 `xmail`：

```toml
[dependencies]
xmail = "0.1.15"
# 如需 TLD 支持：
# xmail = { version = "0.1.15", features = ["tld"] }
```

### 示例代码

```rust
fn main() {
    // 基础标准化
    let email = "  User@Example.COM  ";
    let normalized = xmail::norm(email).unwrap();
    assert_eq!(normalized, "user@example.com");

    // Punycode 解码
    // xn--yfro4i67o 是 "新加坡" 的 Punycode 编码
    let idn_email = "user@site.xn--yfro4i67o";
    let (user, host) = xmail::norm_user_host(idn_email).unwrap();
    assert_eq!(host, "site.新加坡");

    // TLD 提取 (需要 "tld" feature)
    #[cfg(feature = "tld")]
    {
        let (_, tld) = xmail::norm_tld("user@example.co.uk").unwrap();
        assert_eq!(tld, "co.uk");
    }
}
```

## 设计思路
本库通过以下流水线处理邮件地址：
1.  **输入**：接受任何实现了 `AsRef<str>` 的字符串类型。
2.  **拆分**：`user_host` 函数定位第一个 `@` 符号，将地址分为用户和主机两部分。
3.  **标准化**：
    *   去除首尾空白字符。
    *   将字符转换为小写。
    *   遍历域名部分（以 `.` 分隔），使用 Punycode 解码任何以 `xn--` 开头的部分。
4.  **输出**：返回处理后的部分或重组的邮件字符串，并包裹在 `Option` 中。

## 技术栈
- **Rust**：核心语言，提供安全性和高性能。
- **xstr**：用于高效的字符串操作（如修剪、切割）。
- **punycode**：处理国际化域名的解码。
- **xtld**：（可选）基于公共后缀列表（Public Suffix List）提供准确的 TLD 提取。

## 目录结构
```
.
├── Cargo.toml          # 项目配置
├── README.mdt          # README 生成模板
├── readme/             # 文档存储
│   ├── en.md           # 英文文档
│   └── zh.md           # 中文文档
├── src/
│   └── lib.rs          # 核心库逻辑
└── tests/
    └── main.rs         # 集成测试
```

## API 参考

### `user_host(mail: impl AsRef<str>) -> Option<(String, String)>`
将邮件地址拆分为用户和主机部分，不进行标准化。如果出现以下情况则返回 `None`：
- 缺少 `@` 符号。
- 用户名为空。
- 主机名不包含 `.`。
- 主机名以 `.` 开头或结尾。

### `norm_user_host(mail: impl AsRef<str>) -> Option<(String, String)>`
标准化邮件地址（小写、去空），并解码主机部分的 Punycode。

### `norm(mail: impl AsRef<str>) -> Option<String>`
标准化邮件地址并将其作为单个字符串 `user@host` 返回。

### `norm_tld(mail: impl AsRef<str>) -> Option<(String, String)>`
*需要 `tld` 特性。* 标准化邮件地址，并返回包含完整邮件地址和提取出的 TLD 的元组。

## 历史趣闻
电子邮件地址中的 `@` 符号是由 Ray Tomlinson 在 1971 年引入的，用于区分用户和主机。他选择这个符号仅仅因为它是键盘上很少使用的一个字符，而且在语言逻辑上很通顺（"user" at "host"）。

本项目用于处理国际域名的 **Punycode** 一词也有其有趣的起源。它与 "Unicode" 押韵，并且在三个方面体现了 "puny"（微小）的含义：它使用极小的字符集（ASCII），生成短字符串，且实现代码量很小。这种巧妙的编码方式使得非 ASCII 字符（如中文或 Emoji）得以在仅支持 ASCII 的传统 DNS 系统中共存。

---

## 关于

本项目为 [js0.site ⋅ 重构互联网计划](https://js0.site) 的开源组件。

我们正在以组件化的方式重新定义互联网的开发范式，欢迎关注：

* [谷歌邮件列表](https://groups.google.com/g/js0-site)
* [js0site.bsky.social](https://bsky.app/profile/js0site.bsky.social)
