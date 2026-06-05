[English](#en) | [中文](#zh)

---

<a id="en"></a>

# ver_from_txt : Parse version updates from DNS TXT records

<!-- toc -->

## Introduction

`ver_from_txt` is a Rust library designed to parse version update information published in DNS TXT records. It supports decoding base64 encoded version numbers and parsing download URLs, including automatic expansion of GitHub release URLs. This enables applications to efficiently check for updates via DNS protocol.

## Usage

```rust
use aok::{OK, Void};
use log::info;
use ver_from_txt::ver_from_txt;

#[static_init::constructor(0)]
extern "C" fn _loginit() {
  log_init::init();
}

#[test]
fn test() -> Void {
  let txt = "AAEp;Gup51/v;up[0,2~3].u-01.eu.org;yutk.eu.org";

  let r = ver_from_txt("i18", &[0, 0, 1], txt)?;
  info!("{:?}", r);
  OK
}
```

Output:

```
Some(VerUrlLi {
  ver: Ver(0.1.41),
  url_li: [
    "https://github.com/up51/v/releases/download/i18-0.1.41",
    "https://up0.u-01.eu.org/i18/0.1.41",
    "https://up2.u-01.eu.org/i18/0.1.41",
    "https://up3.u-01.eu.org/i18/0.1.41",
    "https://yutk.eu.org/i18/0.1.41"
  ]
})
```

## Design

The library processes the TXT record in the following steps:

1.  **Split & Decode**: The input string is split by `;`. The first part is treated as a Base64 encoded version number.
2.  **Version Comparison**: It decodes the version using `vb` (variable byte) encoding and compares it with the provided current version. If the parsed version is not greater, it returns `None`.
3.  **URL Parsing**: Given the length constraints of DNS TXT records (typically 255 bytes per character string, and keeping total packet size small is preferable), the library uses a compact representation:
    - **GitHub**: Prefixed with `G`, expanded to `https://github.com/...`.
    - **Bracket Expansion**: Supports `[prefix]range` syntax to generate multiple URLs (e.g., for different mirrors).
    - **Standard URL**: Direct URL segments.

## Tech Stack

- **Language**: Rust
- **Dependencies**:
  - `thiserror`: For ergonomic error handling.
  - `base64`: For decoding version strings.
  - `sver`: Semantic versioning support.
  - `vb`: Variable byte decoding.

## Directory Structure

```
.
├── Cargo.toml
├── readme
│   ├── en.md
│   └── zh.md
├── src
│   ├── error.rs    // Error definitions
│   ├── lib.rs      // Core logic
│   └── name_li.rs  // Helper for name list expansion
├── test.sh         // Test runner
└── tests
    └── main.rs     // Integration tests
```

## API Exports

### Structs

- `VerUrlLi`: Contains the new `Ver` and a `Vec<String>` of download URLs.
- `Error`: Enum representing possible errors (Base64 decode, Vb decode, Invalid Text).

### Functions

- `ver_from_txt`: The main entry point.
  ```rust
  pub fn ver_from_txt(project: &str, pre_ver: &[u64; 3], txt: &str) -> Result<Option<VerUrlLi>>
  ```

## History

In the early days of DNS (RFC 1035), TXT records were intended for simple human-readable notes. However, their flexibility soon turned them into the "Swiss Army Knife" of DNS. Anecdotes tell of system administrators using them to store server latitude/longitude "missile coordinates" (University of Edinburgh) or even slicing movies into distributed download links. Today, they are the backbone of modern email security (SPF, DKIM) and domain verification, proving that a simple text field can become a critical infrastructure component.

---

## About

This project is an open-source component of [js0.site ⋅ Refactoring the Internet Plan](https://js0.site).

We are redefining the development paradigm of the Internet in a componentized way. Welcome to follow us:

- [Google Group](https://groups.google.com/g/js0-site)
- [js0site.bsky.social](https://bsky.app/profile/js0site.bsky.social)

---

<a id="zh"></a>

# ver_from_txt : 解析 DNS TXT 记录中的版本更新信息

<!-- toc -->

## 项目简介

`ver_from_txt` 是一个 Rust 库，旨在解析发布在 DNS TXT 记录中的版本更新信息。它支持解码 Base64 编码的版本号及解析下载网址，并具备自动展开 GitHub release 链接的功能。这使得应用程序能够通过 DNS 协议高效地检查更新。

## 使用演示

```rust
use aok::{OK, Void};
use log::info;
use ver_from_txt::ver_from_txt;

#[static_init::constructor(0)]
extern "C" fn _loginit() {
  log_init::init();
}

#[test]
fn test() -> Void {
  let txt = "AAEp;Gup51/v;up[0,2~3].u-01.eu.org;yutk.eu.org";

  let r = ver_from_txt("i18", &[0, 0, 1], txt)?;
  info!("{:?}", r);
  OK
}
```

输出:

```
Some(VerUrlLi {
  ver: Ver(0.1.41),
  url_li: [
    "https://github.com/up51/v/releases/download/i18-0.1.41",
    "https://up0.u-01.eu.org/i18/0.1.41",
    "https://up2.u-01.eu.org/i18/0.1.41",
    "https://up3.u-01.eu.org/i18/0.1.41",
    "https://yutk.eu.org/i18/0.1.41"
  ]
})
```

## 设计思路

库对 TXT 记录的处理流程如下：

1.  **拆分与解码**：输入字符串以 `;` 分隔。第一部分作为 Base64 编码的版本号进行处理。
2.  **版本比对**：使用 `vb` (variable byte) 编码解码版本号，并与提供的当前版本进行比较。如果解析出的版本不大于当前版本，则返回 `None`。
3.  **网址解析**：鉴于 DNS TXT 记录通常有长度限制（单条字符串上限 255 字节，虽然可拼接但过长会导致响应包过大），库采用了紧凑的压缩表示法：
    - **GitHub**：以 `G` 开头，自动展开为 `https://github.com/...` 格式。
    - **括号展开**：支持 `[prefix]range` 语法以生成多个 URL（例如用于多镜像源）。
    - **标准 URL**：直接拼接 URL 片段。

## 技术堆栈

- **语言**：Rust
- **依赖库**：
  - `thiserror`：用于简便的错误定义。
  - `base64`：用于解码版本字符串。
  - `sver`：语义化版本支持。
  - `vb`：变长字节解码。

## 目录结构

```
.
├── Cargo.toml
├── readme
│   ├── en.md
│   └── zh.md
├── src
│   ├── error.rs    // 错误定义
│   ├── lib.rs      // 核心逻辑
│   └── name_li.rs  // 名称列表展开助手
├── test.sh         // 测试脚本
└── tests
    └── main.rs     // 集成测试
```

## 导出接口

### 数据结构

- `VerUrlLi`: 包含新的版本信息 `Ver` 和下载链接列表 `Vec<String>`。
- `Error`: 枚举类型，表示可能的错误（Base64 解码错误、Vb 解码错误、无效文本）。

### 函数

- `ver_from_txt`: 主要入口函数。
  ```rust
  pub fn ver_from_txt(project: &str, pre_ver: &[u64; 3], txt: &str) -> Result<Option<VerUrlLi>>
  ```

## 历史背景

在 DNS 的早期（RFC 1035），TXT 记录仅用于存储简单的人类可读注释。然而，其灵活性很快使其成为了 DNS 中的“瑞士军刀”。坊间趣闻提到，爱丁堡大学的管理员曾用它来存储服务器的经纬度“导弹坐标”，描述为"The world's slowest geography database"。甚至有人尝试利用它将电影切片以构建分布式下载服务。如今，TXT 记录已成为现代电子邮件安全（SPF、DKIM）和域名验证的基石，证明了一个简单的文本字段也能演变成关键的基础设施组件。

---

## 关于

本项目为 [js0.site ⋅ 重构互联网计划](https://js0.site) 的开源组件。

我们正在以组件化的方式重新定义互联网的开发范式，欢迎关注：

- [谷歌邮件列表](https://groups.google.com/g/js0-site)
- [js0site.bsky.social](https://bsky.app/profile/js0site.bsky.social)
