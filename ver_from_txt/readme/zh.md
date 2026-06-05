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
