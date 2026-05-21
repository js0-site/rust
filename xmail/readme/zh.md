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
    - 去除首尾空白字符。
    - 将字符转换为小写。
    - 遍历域名部分（以 `.` 分隔），使用 Punycode 解码任何以 `xn--` 开头的部分。
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

_需要 `tld` 特性。_ 标准化邮件地址，并返回包含完整邮件地址和提取出的 TLD 的元组。

## 历史趣闻

电子邮件地址中的 `@` 符号是由 Ray Tomlinson 在 1971 年引入的，用于区分用户和主机。他选择这个符号仅仅因为它是键盘上很少使用的一个字符，而且在语言逻辑上很通顺（"user" at "host"）。

本项目用于处理国际域名的 **Punycode** 一词也有其有趣的起源。它与 "Unicode" 押韵，并且在三个方面体现了 "puny"（微小）的含义：它使用极小的字符集（ASCII），生成短字符串，且实现代码量很小。这种巧妙的编码方式使得非 ASCII 字符（如中文或 Emoji）得以在仅支持 ASCII 的传统 DNS 系统中共存。
