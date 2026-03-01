[English](#en) | [中文](#zh)

---

<a id="en"></a>

# kind2sql : Type-safe Binary to SQL Converter

A Rust library for efficiently converting a stream of serialized binary data into a sequence of SQL-compatible string values.

## Table of Contents

- [Introduction](#introduction)
- [Features](#features)
- [Usage](#usage)
- [Design](#design)
- [API Reference](#api-reference)
- [Tech Stack](#tech-stack)
- [Directory Structure](#directory-structure)
- [A Little Story](#a-little-story)

## Introduction

`kind2sql` provides a mechanism to map structured binary data to a format ready for SQL `INSERT` statements. It is designed for scenarios where performance is critical, such as bulk-importing logs or other serialized records from a byte stream directly into a database without intermediate parsing into high-level structs.

## Features

- **High Performance**: Operates directly on byte buffers (`Buf`), minimizing allocations and copying.
- **Type-Safe Schema**: Uses an enum (`Kind`) to define the "schema" of the binary data, preventing deserialization errors.
- **Extensible**: Supports different SQL dialects through the `SqlField` trait. A MySQL implementation is provided out-of-the-box.
- **Efficient Encoding**: Uses variable-byte encoding for string and byte array lengths, saving space.
- **Feature Gated**: Core functionality is split into features (`sql_field`, `mysql`) to keep the dependency tree minimal.

## Usage

First, add `kind2sql` to your `Cargo.toml` and enable the desired features. For example, to use the MySQL converter:

```toml
[dependencies]
kind2sql = { version = "0.1.0", features = ["mysql"] }
```

Next, define your data schema with `Kind` and pass your serialized data to the `sql_field` function.

```rust
use kind2sql::{Kind, SqlField, mysql::Mysql};

// 1. Define the data schema.
let kinds = [Kind::U8, Kind::I16, Kind::String];

// 2. Prepare the serialized data buffer.
let mut data = vec![];
// U8: 123
data.push(123u8);
// I16: -456 (little endian)
data.extend_from_slice(&(-456i16).to_le_bytes());
// String: "hello" (length-prefixed)
data.push(5u8); // vbyte encoded length
data.extend_from_slice(b"hello");

// 3. Convert the data.
let result = Mysql::sql_field(&kinds, &data[..]).unwrap();

assert_eq!(result, vec!["123", "-456", "'hello'"]);
```

The output `Vec<String>` contains values that are properly formatted and escaped for a MySQL `INSERT` statement.

## Design

The library's design revolves around three main components:

1.  **`Kind` Enum**: This enum acts as a schema descriptor. An array of `Kind` variants defines the sequence and type of data fields packed in the binary buffer.

2.  **`sql_field` function**: This internal function is the engine of the library. It reads from a type implementing `bytes::Buf` and, based on the `Kind` provided, deserializes one value, converts it to a string, and advances the buffer. For `String` and `Bytes`, it first decodes a variable-byte integer to determine the length of the upcoming data.

3.  **`SqlField` Trait**: This trait abstracts the dialect-specific formatting, particularly for binary data (`BLOB`). To support a new database (e.g., PostgreSQL), you would implement `SqlField` and provide a `blob` function that formats byte arrays according to that database's requirements.

The overall process is:
`SqlField::sql_field` -> loops through `&[Kind]` -> calls internal `sql_field` for each `Kind` -> `sql_field` reads from `Buf`, formats data, and pushes to output vector.

## API Reference

### `enum Kind`

Defines the supported data types that can be deserialized from the byte buffer.

- `U8`, `I8`: 1-byte integers
- `U16`, `I16`: 2-byte little-endian integers
- `U32`: Variable-byte encoded unsigned 32-bit integer (1-5 bytes)
- `I32`: 4-byte little-endian signed integer
- `U64`: Variable-byte encoded unsigned 64-bit integer (1-10 bytes)
- `I64`: 8-byte little-endian signed integer
- `String`: A UTF-8 string, prefixed with its `vbyte`-encoded length
- `Bytes`: A byte array, prefixed with its `vbyte`-encoded length

### `trait SqlField`

A trait to be implemented by dialect-specific converters.

- `fn blob(data: &[u8]) -> String`: A required function that defines how a byte slice should be formatted for the target SQL dialect (e.g., `X'AABBCC'` for MySQL).
- `fn sql_field(...)`: The primary method that orchestrates the conversion process using the provided kinds and buffer.

### `mysql::Mysql`

A concrete implementation of `SqlField` for MySQL, available under the `mysql` feature flag.

## Tech Stack

- [Rust](https://www.rust-lang.org/)
- `bytes`: For efficient buffer manipulation.
- `num_enum`: For converting the `Kind` enum from/to integers.
- `vb`: For `vbyte` encoding/decoding of lengths.
- `sqle`: For SQL string and blob escaping.

## Directory Structure

```
.
├── Cargo.toml      # Package manifest
├── AGENTS.md       # Agent instructions
├── readme/         # Documentation
│   ├── en.md       # English README
│   └── zh.md       # Chinese README
├── src/
│   ├── lib.rs      # Main library file, exports modules and types
│   └── sql_field.rs   # Core conversion logic and `SqlField` trait
└── tests/
    └── main.rs     # Integration tests
```

## A Little Story

The term "BLOB" (Binary Large Object) was coined by Jim Starkey at Digital Equipment Corporation (DEC) in the 1980s. According to Starkey, he had just watched the 1958 horror film *The Blob*, which features a gelatinous, amorphous alien that consumes everything in its path. He thought it was a fitting name for the data type he was working on, which was designed to store large, unstructured chunks of binary data in a database. The name stuck and is now a standard part of SQL, reminding us that even in the structured world of databases, there's a place for a bit of amorphous creativity.

---

## About

This project is an open-source component of [js0.site ⋅ Refactoring the Internet Plan](https://js0.site).

We are redefining the development paradigm of the Internet in a componentized way. Welcome to follow us:

* [Google Group](https://groups.google.com/g/js0-site)
* [js0site.bsky.social](https://bsky.app/profile/js0site.bsky.social)

---

<a id="zh"></a>

# kind2sql : 类型安全的二进制转SQL转换器

本库用于将序列化的二进制数据流高效转换为一系列兼容 SQL 的字符串值。

## 目录

- [简介](#简介)
- [特性](#特性)
- [使用演示](#使用演示)
- [设计思路](#设计思路)
- [API 参考](#api-参考)
- [技术栈](#技术栈)
- [目录结构](#目录结构)
- [技术小故事](#技术小故事)

## 简介

`kind2sql` 提供了一种机制，将结构化的二进制数据映射为可直接用于 SQL `INSERT` 语句的格式。它专为性能至关重要的场景设计，例如将日志或其他序列化记录从字节流批量导入数据库，而无需中间解析为高级结构体。

## 特性

- **高性能**：直接操作字节缓冲区 (`Buf`)，最大限度减少内存分配和拷贝。
- **类型安全**：使用枚举 (`Kind`) 定义二进制数据的“模式”，防止反序列化错误。
- **可扩展**：通过 `SqlField` trait 支持不同的 SQL 方言。开箱即用支持 MySQL。
- **高效编码**：使用变长字节编码处理字符串和字节数组长度，节省空间。
- **按需启用**：核心功能通过特性 (`sql_field`, `mysql`) 拆分，保持依赖树精简。

## 使用演示

首先，在 `Cargo.toml` 中添加 `kind2sql` 并启用所需特性。例如，使用 MySQL 转换器：

```toml
[dependencies]
kind2sql = { version = "0.1.0", features = ["mysql"] }
```

接下来，使用 `Kind` 定义数据模式，并将序列化数据传递给 `sql_field` 函数。

```rust
use kind2sql::{Kind, SqlField, mysql::Mysql};

// 1. 定义数据模式
let kinds = [Kind::U8, Kind::I16, Kind::String];

// 2. 准备序列化数据缓冲区
let mut data = vec![];
// U8: 123
data.push(123u8);
// I16: -456 (小端序)
data.extend_from_slice(&(-456i16).to_le_bytes());
// String: "hello" (长度前缀)
data.push(5u8); // vbyte 编码长度
data.extend_from_slice(b"hello");

// 3. 转换数据
let result = Mysql::sql_field(&kinds, &data[..]).unwrap();

assert_eq!(result, vec!["123", "-456", "'hello'"]);
```

输出的 `Vec<String>` 包含已针对 MySQL `INSERT` 语句正确格式化和转义的值。

## 设计思路

本库的设计围绕三个主要组件：

1.  **`Kind` 枚举**：作为模式描述符。`Kind` 变体数组定义了打包在二进制缓冲区中数据字段的顺序和类型。

2.  **`sql_field` 函数**：库的核心引擎。它从实现 `bytes::Buf` 的类型中读取数据，根据提供的 `Kind` 反序列化值，将其转换为字符串，并推进缓冲区。对于 `String` 和 `Bytes`，它首先解码变长整数以确定后续数据的长度。

3.  **`SqlField` Trait**：抽象了特定方言的格式化逻辑，特别是针对二进制数据 (`BLOB`)。如需支持新数据库（例如 PostgreSQL），只需实现 `SqlField` 并提供 `blob` 函数，按照该数据库的要求格式化字节数组。

整体流程为：
`SqlField::sql_field` -> 遍历 `&[Kind]` -> 对每个 `Kind` 调用内部 `sql_field` -> `sql_field` 从 `Buf` 读取、格式化数据并推入输出向量。

## API 参考

### `enum Kind`

定义可从字节缓冲区反序列化的支持数据类型。

- `U8`、`I8`：1 字节整数
- `U16`、`I16`：2 字节小端序整数
- `U32`：变长字节编码无符号 32 位整数（1-5 字节）
- `I32`：4 字节小端序有符号整数
- `U64`：变长字节编码无符号 64 位整数（1-10 字节）
- `I64`：8 字节小端序有符号整数
- `String`：UTF-8 字符串，前置 `vbyte` 编码长度
- `Bytes`：字节数组，前置 `vbyte` 编码长度

### `trait SqlField`

由特定方言转换器实现的 trait。

- `fn blob(data: &[u8]) -> String`：必须实现的函数，定义字节切片应如何为目标 SQL 方言进行格式化（例如 MySQL 的 `X'AABBCC'`）。
- `fn sql_field(...)`：编排整个转换过程的主要方法，使用给定的模式和缓冲区。

### `mysql::Mysql`

`SqlField` 的具体实现，用于 MySQL，在 `mysql` 功能标志下可用。

## 技术栈

- [Rust](https://www.rust-lang.org/)
- `bytes`：用于高效缓冲区操作。
- `num_enum`：用于 `Kind` 枚举与整数间转换。
- `vb`：用于 `vbyte` 长度编码/解码。
- `sqle`：用于 SQL 字符串和 blob 转义。

## 目录结构

```
.
├── Cargo.toml      # 包配置
├── AGENTS.md       # Agent 指示
├── readme/         # 文档
│   ├── en.md       # 英文 README
│   └── zh.md       # 中文 README
├── src/
│   ├── lib.rs      # 库主文件，导出模块和类型
│   └── sql_field.rs   # 核心转换逻辑和 `SqlField` trait
└── tests/
    └── main.rs     # 集成测试
```

## 技术小故事

术语 "BLOB"（二进制大对象）是 Jim Starkey 在 20 世纪 80 年代于数字设备公司（DEC）工作时创造的。据 Starkey 所述，他当时刚看完 1958 年的恐怖电影《The Blob》（一译《幽浮魔点》），影片中有一只无定形的凝胶状外星生物，吞噬其路径上的一切。他认为这个名字非常适合他正在开发的数据类型，该类型旨在数据库中存储非结构化的大块二进制数据。这个名字流传了下来，并成为 SQL 的标准部分，提醒着我们，即使在数据库这样结构化的世界里，也总有非晶体般创造力的空间。

---

## 关于

本项目为 [js0.site ⋅ 重构互联网计划](https://js0.site) 的开源组件。

我们正在以组件化的方式重新定义互联网的开发范式，欢迎关注：

* [谷歌邮件列表](https://groups.google.com/g/js0-site)
* [js0site.bsky.social](https://bsky.app/profile/js0site.bsky.social)
