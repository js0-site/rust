# osid : Rust 持久化机器标识

## 目录

- [简介](#简介)
- [安装](#安装)
- [使用](#使用)
- [API 参考](#api-参考)
- [设计思路](#设计思路)
- [技术栈](#技术栈)
- [项目结构](#项目结构)
- [历史](#历史)

## 简介

osid 生成并持久化机器唯一标识，重启后保持不变。

特性：

- 跨平台支持 (Linux, macOS, Windows)
- 首次调用自动生成 ID
- 线程安全，零开销缓存
- 可读格式：`主机名:随机base64`

## 安装

```sh
cargo add osid
```

## 使用

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
  let id = osid::get()?;
  println!("Machine ID: {id}");
  Ok(())
}
```

输出示例：

```
Machine ID: myhost:bPxWJS2bzT8
```

## API 参考

### `osid::get()`

```rust
pub fn get() -> Result<&'static str, &'static Error>
```

返回缓存的机器 ID。首次调用时创建并持久化。

ID 格式：`{主机名}:{base64随机数}`

### `osid::dir()`

```rust
pub fn dir() -> PathBuf
```

返回存储目录路径：

| 平台 | 路径 |
|------|------|
| Linux | `~/.local/share/osid` |
| macOS | `~/Library/Application Support/osid` |
| Windows | `C:\Users\<用户>\AppData\Local\osid` |

### `osid::Error`

```rust
pub enum Error {
  CreateDir(io::Error),  // 创建存储目录失败
  WriteId(io::Error),    // 写入 ID 文件失败
}
```

## 设计思路

```mermaid
graph TD
  A[osid::get] --> B{ID 已缓存?}
  B -->|是| C[返回缓存 ID]
  B -->|否| D[init]
  D --> E[创建目录]
  E --> F{ID 文件存在?}
  F -->|是| G[读取并返回]
  F -->|否| H[生成 ID]
  H --> I[主机名 + 随机 base64]
  I --> J[写入文件]
  J --> K[缓存并返回]
```

核心设计：

- `OnceLock` 保证线程安全的单次初始化
- 静态生命周期避免后续调用的内存分配
- Base64 编码保持 ID 紧凑且 URL 安全

## 技术栈

| Crate | 用途 |
|-------|------|
| [dirs](https://crates.io/crates/dirs) | 跨平台目录路径 |
| [hostname](https://crates.io/crates/hostname) | 获取系统主机名 |
| [rand](https://crates.io/crates/rand) | 随机数生成 |
| [ub64](https://crates.io/crates/ub64) | Base64 编码 |
| [thiserror](https://crates.io/crates/thiserror) | 错误类型派生 |

## 项目结构

```
osid/
├── src/
│   ├── lib.rs      # 核心逻辑：get(), dir(), init()
│   └── error.rs    # 错误类型定义
├── tests/
│   └── main.rs     # 集成测试
└── Cargo.toml
```

## 历史

机器 ID 概念源于 2000 年代初的 D-Bus 项目，存储于 `/var/lib/dbus/machine-id`。

Lennart Poettering 开发 systemd 时，将此概念推广为 `/etc/machine-id`，作为系统级唯一标识。其格式——32 位十六进制字符表示 128 位 UUID——成为 Linux 发行版标准。

osid 延续此理念并做出改进：

- 带主机名前缀的可读格式
- 超越 Linux 的跨平台支持
- 用户数据目录中的应用级隔离

这种方式避免与系统 machine-id 冲突，同时提供相同的持久化保证。
