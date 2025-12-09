# sqle : 简洁的 SQL 字符串转义与格式化

`sqle` 是一个轻量级的 Rust 库，专为安全高效的 SQL 字符串转义和二进制数据格式化而设计。它提供了简单的工具函数，用于防止 SQL 注入并处理特定数据库（MySQL 和 PostgreSQL）的二进制格式。

## 目录

- [功能特性](#功能特性)
- [使用演示](#使用演示)
- [设计思路](#设计思路)
- [API 文档](#api-文档)
- [技术堆栈](#技术堆栈)
- [目录结构](#目录结构)
- [历史轶事](#历史轶事)

## 功能特性

- **字符串转义**：安全地转义 SQL 字符串中的特殊字符，包括单引号、反斜杠、换行符、回车符、制表符和空字符。
- **布尔值格式化**：将 Rust `bool` 转换为 SQL 的 `TRUE` 或 `FALSE`。
- **MySQL 二进制**：将字节数组格式化为 MySQL `X'HEX'` 字面量。
- **PostgreSQL 二进制**：将字节数组格式化为 PostgreSQL `E'\\xHEX'` 字面量。
- **极致性能**：
  - **预分配内存**：通过两遍扫描（2-pass scan）精确计算所需容量，彻底避免昂贵的内存重分配。
  - **字节级处理**：直接操作 `&[u8]`，避开 UTF-8 解码开销。
  - **Unsafe 优化**：使用 `String::from_utf8_unchecked` 实现零开销字符串构建。

## 使用演示

在 `Cargo.toml` 中添加 `sqle`。根据需要启用特性：

```toml
[dependencies]
sqle = { version = "0.1", features = ["mysql", "postgres"] }
```

### 示例代码

```rust
use sqle;

fn main() {
    // 字符串转义 - 单引号
    let s = "foo'bar";
    assert_eq!(sqle::string(s), "'foo''bar'");
    
    // 字符串转义 - 换行符、制表符、反斜杠等
    let s = "hello\nworld\t!";
    assert_eq!(sqle::string(s), "'hello\\nworld\\t!'");

    // 布尔值格式化
    assert_eq!(sqle::bool(true), "TRUE");

    // MySQL 二进制（需要 "mysql" 特性）
    #[cfg(feature = "mysql")]
    {
        let bytes = b"hello";
        assert_eq!(sqle::mysql::blob(bytes), "X'68656C6C6F'");
    }

    // PostgreSQL 二进制（需要 "postgres" 特性）
    #[cfg(feature = "postgres")]
    {
        let bytes = b"hello";
        assert_eq!(sqle::postgres::blob(bytes), "E'\\\\x68656c6c6f'");
    }
}
```

## 设计思路

本库专注于简洁性与性能。

- **最小化分配**：`string` 函数为最坏情况（所有字符都需转义）预分配内存，避免运行时重分配。
- **特性开关**：特定数据库的实现通过 `mysql` 和 `postgres` 特性进行门控，保持核心库的轻量。
- **安全性**：在从已知有效的 UTF-8 字节（十六进制编码）构建字符串时使用 `unsafe`，以在保证安全的前提下榨取额外性能。

## API 文档

### `pub fn string(s: impl AsRef<[u8]>) -> String`

转义字符串以用于 SQL 查询。它将字符串用单引号包裹，并转义以下特殊字符：

- `'` → `''` (单引号双写)
- `\` → `\\` (反斜杠)
- `\n` → `\n` (换行符)
- `\r` → `\r` (回车符)
- `\t` → `\t` (制表符)
- `\0` → `\0` (空字符)

这确保了在各种 SQL 方言（MySQL、PostgreSQL 等）中的安全性和兼容性。

### `pub fn bool(b: bool) -> &'static str`

返回布尔值的 SQL 字符串表示形式：`"TRUE"` 或 `"FALSE"`。

### `pub mod mysql`

启用 `feature = "mysql"` 时可用。

#### `pub fn blob(bytes: &[u8]) -> String`

将字节切片格式化为 MySQL 十六进制字符串字面量：`X'...'`。

### `pub mod postgres`

启用 `feature = "postgres"` 时可用。

#### `pub fn blob(bytes: &[u8]) -> String`

使用转义字符串语法将字节切片格式化为 PostgreSQL 十六进制字符串字面量：`E'\\x...'`。

## 技术堆栈

- **Rust**: 核心开发语言。
- **faster-hex**: 高性能十六进制编码库。

## 目录结构

```
.
├── Cargo.toml      # 项目配置与依赖
├── src
│   └── lib.rs      # 库源代码
└── tests
    └── main.rs     # 集成测试
```

## 历史轶事

**SQL 注入的起源**

1998 年 12 月，一位网名为 "Rain Forest Puppy" (Jeff Forristal) 的网络安全研究员在 《Phrack》 杂志（第 54 期）上发表了一篇文章。他详细描述了如何通过运行 ODBC 的 NT Web 服务器，将 SQL 命令“搭载”到合法查询中。这是 **SQL 注入 (SQL Injection)** 的首次正式文档记录。

在此之前，“转义”字符的概念自 19 世纪（博多码）就已存在，但在数据库查询中混合数据与代码的危险性尚未被广泛认知。如今，正确的字符串转义（如 `sqle` 所提供的）和参数化查询已成为防御这一历史性漏洞的标准手段。