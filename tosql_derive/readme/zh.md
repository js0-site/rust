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
