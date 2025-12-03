# to_mysql : 高效 MySQL SQL 生成

`to_mysql` 是一个 Rust 库，旨在高效地从 Rust 结构体生成 MySQL `CREATE TABLE` 和 `INSERT` 语句。它利用 `tosql` 和 `kind2sql` 将 Rust 类型映射到 MySQL 类型并序列化数据，为 SQL 生成提供了高性能解决方案。

## 目录

- [简介](#简介)
- [特性](#特性)
- [使用方法](#使用方法)
- [设计思路](#设计思路)
- [技术栈](#技术栈)
- [目录结构](#目录结构)
- [API 文档](#api-文档)
- [历史](#历史)

## 简介

`to_mysql` 通过自动化 SQL 语句生成，简化了与 MySQL 数据库交互的过程。它接收 Rust 结构体的模式定义和数据，并生成优化后的 SQL 字符串，可直接用于执行。

## 特性

- **自动模式生成**：根据结构体字段和类型创建 `CREATE TABLE` 语句。
- **高效数据插入**：生成带有预计算前缀的 `INSERT` 语句，以实现最高性能。
- **类型安全**：确保 Rust 类型正确映射到 MySQL 类型。
- **缓存**：缓存 `INSERT` 语句前缀，减少重复插入时的开销。

## 使用方法

将 `to_mysql` 添加到 `Cargo.toml`。

```rust
use to_mysql::Mysql;
use tosql::{ToSqlTrait, ToSql};

#[derive(ToSql)]
struct User {
  id: u64,
  name: String,
}

fn main() {
  // 生成 CREATE TABLE 语句
  let mysql = Mysql::new("User", User::meta());
  let create_table_sql = mysql.create_table();
  println!("{}", create_table_sql);
  // 输出: CREATE TABLE User(id BIGINT UNSIGNED,name LONGTEXT);

  // 生成 INSERT 语句
  let user = User {
    id: 123,
    name: "Alice".to_string(),
  };
  let insert_sql = mysql.insert(&user.dump()).unwrap();
  println!("{}", insert_sql);
  // 输出: INSERT INTO User(id,name)VALUES(123,'Alice');
}
```

## 设计思路

`to_mysql` 的核心是 `Mysql` 结构体。

1.  **初始化 (`new`)**：创建 `Mysql` 实例时，它接收表名和模式信息（类型和字段名）。它会预计算 `INSERT INTO ... VALUES(` 前缀字符串。这种优化避免了在每次插入时重建 SQL 查询的静态部分。
2.  **模式映射**：`create_table` 方法遍历字段名和类型，将每个 Rust 类型（通过 `Kind`）映射到其对应的 MySQL 类型字符串（例如 `u64` -> `BIGINT UNSIGNED`）。
3.  **数据序列化**：`insert` 方法接收序列化的字节数据，使用 `kind2sql` 将其转换为兼容 SQL 的字符串值，并将它们追加到预计算的前缀后。

## 技术栈

-   **Rust**：核心编程语言。
-   **tosql**：提供 `ToSql` trait 和 `ToSqlTrait` 用于结构体序列化和元数据。
-   **kind2sql**：处理 Rust 类型 (`Kind`) 和 SQL 类型之间的映射，以及值格式化。

## 相关库

- [tosql](https://docs.rs/crate/tosql)：核心 trait 定义。
- [tosql_derive](https://docs.rs/crate/tosql_derive)：用于派生 `ToSqlTrait` 的宏。
- [tosql_meta](https://docs.rs/crate/tosql_meta)：SQL 结构体的元数据定义。

## 目录结构

```
to_mysql/
├── Cargo.toml      # 项目配置和依赖
├── src/
│   └── lib.rs      # 核心逻辑和 Mysql 结构体实现
└── tests/
    └── main.rs     # 展示用法的集成测试
```

## API 文档

### `Mysql` 结构体

库的主要入口点。

#### `pub fn new(table_name: impl Into<String>, (kind_li, field_li): (Vec<Kind>, Vec<String>)) -> Self`

创建一个新的 `Mysql` 实例。
-   `table_name`: MySQL 表的名称。
-   `kind_li`: 代表字段类型的 `Kind` 枚举向量。
-   `field_li`: 代表字段名称的字符串向量。

#### `pub fn create_table(&self) -> String`

生成 `CREATE TABLE` SQL 语句。
-   返回: 包含 SQL 语句的字符串。

#### `pub fn insert(&self, bytes: &[u8]) -> tosql::vb::Result<String>`

为特定记录生成 `INSERT` SQL 语句。
-   `bytes`: 结构体的序列化字节表示（来自 `ToSqlTrait::dump`）。
-   返回: 包含 SQL 字符串或错误的 `Result`。

## 历史

**"MySQL" 名称的由来**

MySQL 由瑞典公司 MySQL AB 创建，该公司由 David Axmark、Allan Larsson 和 Michael "Monty" Widenius 创立。MySQL 中的 "My" 是以 Monty 的女儿 My 命名的。名为 "Sakila" 的海豚标志是从用户在 "为海豚命名" 比赛中建议的大量名字中选出的。该项目始于 1994 年，第一个版本于 1995 年 5 月 23 日发布。它的设计初衷是作为现有数据库系统（如 mSQL）的一种更快、更灵活的替代方案。