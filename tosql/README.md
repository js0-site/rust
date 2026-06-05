[English](#en) | [中文](#zh)

---

<a id="en"></a>

# tosql : Trait for SQL Struct Serialization

Defines the `ToSqlTrait` trait for serializing Rust structs into binary formats compatible with SQL bulk inserts.

## Table of Contents

- [Introduction](#introduction)
- [Features](#features)
- [Usage](#usage)
- [API Reference](#api-reference)
- [Tech Stack](#tech-stack)
- [Directory Structure](#directory-structure)

## Introduction

`tosql` provides the fundamental `ToSqlTrait` trait used to map Rust structs to SQL table schemas. It works in conjunction with `kind2sql` to enable efficient, type-safe serialization of data for database operations.

## Features

- **Standardized Trait**: Defines a common interface for SQL-compatible structs.
- **Type Safety**: Leverages `kind2sql::Kind` to ensure data types match the database schema.
- **Zero-Copy Friendly**: Designed to work with `bytes::Bytes` for efficient memory usage.

## Usage

The most common way to use `tosql` is with the `tosql_derive` macro to automatically implement the `ToSqlTrait` trait.

First, add dependencies:

```toml
[dependencies]
tosql = "0.1"
tosql_derive = "0.1"
```

Then, derive `ToSql` for your structs:

```rust
use tosql::{SqlField, ToSqlTrait, ToSql, mysql::{KIND, Mysql}};

#[derive(ToSql, Debug)]
struct User {
  id: u64,
  name: String,
  age: u8,
}

fn main() {
  let user = User {
    id: 1001,
    name: "Alice".to_string(),
    age: 30,
  };

  // 1. Get Schema Information
  println!("Table Name: {}", User::name());
  println!("Fields: {:?}", User::field_li());
  println!("Types: {:?}", User::kind_li());

  // 2. Serialize Data
  let bytes = user.dump();
  println!("Serialized bytes: {:?}", bytes);

  // 3. Convert to SQL Values (using kind2sql's Mysql implementation)
  let sql_values = Mysql::sql_field(&User::kind_li(), bytes).unwrap();
  println!("SQL Values: {:?}", sql_values);
  // Output: ["1001", "'Alice'", "30"]

  // 4. Generate SQL Statement Example
  let columns = User::field_li().join(", ");
  let values = sql_values.join(", ");
  println!("INSERT INTO `{}` ({}) VALUES ({});", User::name(), columns, values);
}
```

## API Reference

### `trait ToSqlTrait`

- `fn name() -> String`: Returns the struct (or table) name.
- `fn kind_li() -> Vec<Kind>`: Returns the list of field types (`Kind`).
- `fn field_li() -> Vec<String>`: Returns the list of field names.
- `fn dump(&self) -> Bytes`: Serializes the struct instance into a binary buffer.

## Tech Stack

- [Rust](https://www.rust-lang.org/)
- `bytes`: For efficient byte buffer management.
- `kind2sql`: For type definitions (`Kind`).

## Related Crates

- [tosql_derive](https://docs.rs/crate/tosql_derive): Macro to derive `ToSqlTrait`.
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
    └── lib.rs      # Trait definition
```

---

## About

This project is an open-source component of [js0.site ⋅ Refactoring the Internet Plan](https://js0.site).

We are redefining the development paradigm of the Internet in a componentized way. Welcome to follow us:

- [Google Group](https://groups.google.com/g/js0-site)
- [js0site.bsky.social](https://bsky.app/profile/js0site.bsky.social)

---

<a id="zh"></a>

# tosql : SQL结构体序列化Trait

定义了 `ToSqlTrait` trait，用于将 Rust 结构体序列化为兼容 SQL 批量插入的二进制格式。

## 目录

- [简介](#简介)
- [特性](#特性)
- [使用演示](#使用演示)
- [API 参考](#api-参考)
- [技术栈](#技术栈)
- [目录结构](#目录结构)

## 简介

`tosql` 提供了基础的 `ToSqlTrait` trait，用于将 Rust 结构体映射到 SQL 表模式。它与 `kind2sql` 配合使用，实现数据库操作的高效、类型安全的数据序列化。

## 特性

- **标准化接口**：为兼容 SQL 的结构体定义通用接口。
- **类型安全**：利用 `kind2sql::Kind` 确保数据类型与数据库模式匹配。
- **零拷贝友好**：设计用于配合 `bytes::Bytes` 使用，实现高效内存管理。

## 使用演示

使用 `tosql` 最常见的方式是配合 `tosql_derive` 宏自动实现 `ToSqlTrait` trait。

首先，添加依赖：

```toml
[dependencies]
tosql = "0.1"
tosql_derive = "0.1"
```

然后，为你的结构体派生 `ToSql`：

```rust
use tosql::{SqlField, ToSqlTrait, ToSql, mysql::{KIND, Mysql}};

#[derive(ToSql, Debug)]
struct User {
  id: u64,
  name: String,
  age: u8,
}

fn main() {
  let user = User {
    id: 1001,
    name: "Alice".to_string(),
    age: 30,
  };

  // 1. 获取模式信息
  println!("表名: {}", User::name());
  println!("字段: {:?}", User::field_li());
  println!("类型: {:?}", User::kind_li());

  // 2. 序列化数据
  let bytes = user.dump();
  println!("序列化字节: {:?}", bytes);

  // 3. 转换为 SQL 值 (使用 kind2sql 的 Mysql 实现)
  let sql_values = Mysql::sql_field(&User::kind_li(), bytes).unwrap();
  println!("SQL 值: {:?}", sql_values);
  // 输出: ["1001", "'Alice'", "30"]

  // 4. 生成 SQL 语句示例
  let columns = User::field_li().join(", ");
  let values = sql_values.join(", ");
  println!("INSERT INTO `{}` ({}) VALUES ({});", User::name(), columns, values);
}
```

## API 参考

### `trait ToSqlTrait`

- `fn name() -> String`：返回结构体（或表）名称。
- `fn kind_li() -> Vec<Kind>`：返回字段类型列表 (`Kind`)。
- `fn field_li() -> Vec<String>`：返回字段名称列表。
- `fn dump(&self) -> Bytes`：将结构体实例序列化为二进制缓冲区。

## 技术栈

- [Rust](https://www.rust-lang.org/)
- `bytes`：用于高效字节缓冲区管理。
- `kind2sql`：用于类型定义 (`Kind`)。

## 相关库

- [tosql_derive](https://docs.rs/crate/tosql_derive)：用于派生 `ToSqlTrait` 的宏。
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
    └── lib.rs      # Trait 定义
```

---

## 关于

本项目为 [js0.site ⋅ 重构互联网计划](https://js0.site) 的开源组件。

我们正在以组件化的方式重新定义互联网的开发范式，欢迎关注：

- [谷歌邮件列表](https://groups.google.com/g/js0-site)
- [js0site.bsky.social](https://bsky.app/profile/js0site.bsky.social)
