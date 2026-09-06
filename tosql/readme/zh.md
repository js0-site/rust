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
