[English](#en) | [中文](#zh)

---

<a id="en"></a>

# tosql_derive : Derive Macro for tosql

Automatically implements the `ToSqlTrait` trait for Rust structs, simplifying SQL serialization.

## Table of Contents

- [Introduction](#introduction)
- [Features](#features)
- [Usage](#usage)
- [Design](#design)
- [Tech Stack](#tech-stack)
- [Directory Structure](#directory-structure)

## Introduction

`tosql_derive` is a procedural macro that generates the boilerplate code required to implement the `tosql::ToSqlTrait` trait. It analyzes your struct fields and generates the appropriate serialization logic, mapping Rust types to `kind2sql::Kind` variants.

## Features

- **Automatic Derivation**: Simply add `#[derive(ToSql)]` to your struct.
- **Type Mapping**: Automatically maps Rust primitive types (`u8`, `i32`, `String`, etc.) to SQL types.
- **Variable Byte Encoding**: Handles efficient encoding for `String`, `Bytes`, `u32`, and `u64`.
- **Seamless Integration**: Designed to work perfectly with the `tosql` and `kind2sql` ecosystem.

## Usage

Add `tosql` and `tosql_derive` to your dependencies:

```toml
[dependencies]
tosql = "0.1"
tosql_derive = "0.1"
```

Then derive `ToSql` on your struct:

```rust
use tosql_derive::ToSql;

#[derive(ToSql)]
struct User {
    id: u64,
    username: String,
    is_active: u8,
}

// Now User implements ToSqlTrait automatically
// You can use it with kind2sql to generate SQL inserts
```

## Design

The macro parses the struct definition and performs the following mappings:

- **Integers**: Mapped to their corresponding `Kind` (e.g., `u8` -> `Kind::U8`). `u32` and `u64` are treated as variable-byte encoded integers to save space.
- **Strings**: Mapped to `Kind::String`. The length is encoded using variable-byte encoding before the string data.
- **Bytes**: `Vec<u8>`, `Bytes`, and `bytes::Bytes` are mapped to `Kind::Bytes`.

The generated `dump` method efficiently writes these values into a `BytesMut` buffer without unnecessary copying.

## Tech Stack

- [Rust](https://www.rust-lang.org/)
- `syn`: For parsing Rust code.
- `quote`: For generating Rust code.
- `proc-macro2`: For manipulating token streams.

## Related Crates

- [tosql](https://docs.rs/crate/tosql): The core trait definition.
- [to_mysql](https://docs.rs/crate/to_mysql): MySQL SQL generation logic.
- [tosql_meta](https://docs.rs/crate/tosql_meta): Metadata definition for SQL structs.

## Directory Structure

```
.
├── Cargo.toml      # Package manifest
├── readme/         # Documentation
│   ├── en.md       # English README
│   └── zh.md       # Chinese README
└── src/
    └── lib.rs      # Macro implementation
```

---

## About

This project is an open-source component of [js0.site ⋅ Refactoring the Internet Plan](https://js0.site).

We are redefining the development paradigm of the Internet in a componentized way. Welcome to follow us:

- [Google Group](https://groups.google.com/g/js0-site)
- [js0site.bsky.social](https://bsky.app/profile/js0site.bsky.social)

---

<a id="zh"></a>

# tosql_derive : tosql 的派生宏

自动为 Rust 结构体实现 `ToSqlTrait` trait，简化 SQL 序列化过程。

## 目录

- [简介](#简介)
- [特性](#特性)
- [使用演示](#使用演示)
- [设计思路](#设计思路)
- [技术栈](#技术栈)
- [目录结构](#目录结构)

## 简介

`tosql_derive` 是一个过程宏，用于生成实现 `tosql::ToSqlTrait` trait 所需的样板代码。它分析结构体字段并生成相应的序列化逻辑，将 Rust 类型映射到 `kind2sql::Kind` 变体。

## 特性

- **自动派生**：只需在结构体上添加 `#[derive(ToSql)]`。
- **类型映射**：自动将 Rust 基本类型（`u8`、`i32`、`String` 等）映射到 SQL 类型。
- **变长字节编码**：处理 `String`、`Bytes`、`u32` 和 `u64` 的高效编码。
- **无缝集成**：设计用于与 `tosql` 和 `kind2sql` 生态系统完美配合。

## 使用演示

在依赖项中添加 `tosql` 和 `tosql_derive`：

```toml
[dependencies]
tosql = "0.1"
tosql_derive = "0.1"
```

然后在结构体上派生 `ToSql`：

```rust
use tosql_derive::ToSql;

#[derive(ToSql)]
struct User {
    id: u64,
    username: String,
    is_active: u8,
}

// 现在 User 自动实现了 ToSqlTrait
// 你可以将其与 kind2sql 结合使用以生成 SQL 插入语句
```

## 设计思路

该宏解析结构体定义并执行以下映射：

- **整数**：映射到相应的 `Kind`（例如 `u8` -> `Kind::U8`）。`u32` 和 `u64` 被视为变长字节编码整数以节省空间。
- **字符串**：映射到 `Kind::String`。在字符串数据之前使用变长字节编码对长度进行编码。
- **字节**：`Vec<u8>`、`Bytes` 和 `bytes::Bytes` 映射到 `Kind::Bytes`。

生成的 `dump` 方法将这些值高效地写入 `BytesMut` 缓冲区，无需不必要的复制。

## 技术栈

- [Rust](https://www.rust-lang.org/)
- `syn`：用于解析 Rust 代码。
- `quote`：用于生成 Rust 代码。
- `proc-macro2`：用于操作令牌流。

## 相关库

- [tosql](https://docs.rs/crate/tosql)：核心 trait 定义。
- [to_mysql](https://docs.rs/crate/to_mysql)：MySQL SQL 生成逻辑。
- [tosql_meta](https://docs.rs/crate/tosql_meta)：SQL 结构体的元数据定义。

## 目录结构

```
.
├── Cargo.toml      # 包配置
├── readme/         # 文档
│   ├── en.md       # 英文 README
│   └── zh.md       # 中文 README
└── src/
    └── lib.rs      # 宏实现
```

---

## 关于

本项目为 [js0.site ⋅ 重构互联网计划](https://js0.site) 的开源组件。

我们正在以组件化的方式重新定义互联网的开发范式，欢迎关注：

- [谷歌邮件列表](https://groups.google.com/g/js0-site)
- [js0site.bsky.social](https://bsky.app/profile/js0site.bsky.social)
