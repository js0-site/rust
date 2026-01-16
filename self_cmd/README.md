[English](#en) | [中文](#zh)

---

<a id="en"></a>

# self_cmd: Self-Executing Command Builder

- [Overview](#overview)
- [Features](#features)
- [Usage](#usage)
- [Design Philosophy](#design-philosophy)
- [Technical Stack](#technical-stack)
- [Project Structure](#project-structure)
- [API Reference](#api-reference)
- [Error Handling](#error-handling)

## Overview

`self_cmd` is a Rust library that creates Command instances for re-executing the current program with identical arguments and stdio inheritance. On Linux, it provides additional support for systemd socket activation by preserving file descriptor mappings through the `LISTEN_FDS` protocol.

## Features

- **Self-Replication**: Generate Command objects that execute the current program
- **Argument Preservation**: Automatically captures and forwards command-line arguments
- **Stdio Inheritance**: Maintains stdin, stdout, and stderr connections
- **Socket Activation Support**: Linux-specific file descriptor mapping for systemd integration
- **Robust Error Handling**: Custom error types with proper error propagation
- **Logging Integration**: Built-in logging support for debugging and monitoring

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
self_cmd = "0.1.7"
```

```rust
use self_cmd;

fn main() -> self_cmd::Result<()> {
    // Create and execute a command that re-runs this program
    // Automatically handles stdio inheritance and Linux FD mappings for systemd socket activation
    let mut cmd = self_cmd::get()?;
    let status = cmd.status()?;
    println!("Child process exited with: {status}");
    
    Ok(())
}
```

On Linux systems, the library automatically handles systemd socket activation by checking `LISTEN_FDS`, duplicating file descriptors starting from FD 3, and configuring the child process environment. This makes it seamless for services that need to restart while maintaining their listening sockets.

## Design Philosophy

The library follows a minimalist approach with a single public function that encapsulates the complexity of self-execution:

```mermaid
graph TD
    A[Program Start] --> B["self_cmd::get()"]
    B --> C["env::current_exe()"]
    C --> D["env::args().skip(1)"]
    D --> E["Command::new(program)"]
    E --> F["cmd.args(args)"]
    F --> G[Configure Stdio Inheritance]
    G --> H{Linux?}
    H -->|Yes| I["fd_mapping()"]
    I --> J[Check LISTEN_FDS env]
    J --> K[Duplicate FDs 3+]
    K --> L[Create FdMapping]
    L --> M[Set LISTEN_FDS for child]
    M --> N[Remove LISTEN_PID]
    N --> O[Return Command]
    H -->|No| O
    O --> P[Execute Command]
```

The design ensures:
- **Simplicity**: Single function interface
- **Reliability**: Proper error handling without panics
- **Flexibility**: Returns Command for further customization
- **Performance**: Minimal overhead with lazy evaluation

## Technical Stack

- **Language**: Rust 2024 Edition
- **Core Dependencies**: 
  - `log` for logging functionality
  - `thiserror` for structured error handling
- **Linux Dependencies**:
  - `command-fds` for file descriptor mapping
  - `libc` for low-level system calls
- **Error Handling**: Custom `Error` type with `thiserror` integration
- **Process Management**: `std::process::Command` with FD mapping extensions
- **Environment Access**: `std::env` for executable path and arguments

## Project Structure

```
self_cmd/
├── src/
│   ├── lib.rs          # Core implementation and public API
│   ├── error.rs        # Error types and handling
│   └── fd_mapping.rs   # Linux-specific FD mapping (conditional)
├── tests/
│   └── main.rs         # Integration tests
├── readme/
│   ├── en.md          # English documentation
│   └── zh.md          # Chinese documentation
├── Cargo.toml         # Project configuration
└── test.sh           # Test runner script
```

## API Reference

### `get() -> Result<Command>`

Creates a Command instance configured to re-execute the current program with full context preservation.

**Returns:**
- `Ok(Command)`: Configured command ready for execution
- `Err(Error)`: If unable to determine executable path or configure FD mappings

**Behavior:**
- Captures current executable path via `env::current_exe()`
- Preserves all command-line arguments except program name
- Configures stdio inheritance (stdin, stdout, stderr)
- **Linux only**: Handles systemd socket activation FD mapping
- Returns Command for further customization before execution

**Example:**
```rust
match self_cmd::get() {
    Ok(mut cmd) => {
        // Command is ready to use with full context preservation
        cmd.status()?;
    }
    Err(e) => {
        eprintln!("Failed to create self command: {e}");
    }
}
```

### `fd_mapping() -> Result<Vec<FdMapping>>` (Linux only)

**Available on Linux only** - Creates file descriptor mappings for systemd socket activation.

**Returns:**
- `Ok(Vec<FdMapping>)`: List of FD mappings for socket activation
- `Err(Error)`: If FD duplication or validation fails

This function is automatically called by `get()` on Linux systems and typically doesn't need to be used directly.

## Error Handling

The library uses a custom `Error` type built with `thiserror` for comprehensive error handling:

```rust
use self_cmd::{Error, Result};

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("FD mapping collision: {0}")]
    FdMappingCollision(#[from] command_fds::FdMappingCollision),
}
```

All public functions return `Result<T>` which is an alias for `std::result::Result<T, Error>`. This provides:

- **Structured Errors**: Clear error types for different failure modes
- **Error Chaining**: Automatic conversion from underlying error types
- **Debugging Support**: Rich error messages with context
- **Integration**: Seamless integration with `?` operator and error handling patterns

---

## About

This project is an open-source component of [js0.site ⋅ Refactoring the Internet Plan](https://js0.site).

We are redefining the development paradigm of the Internet in a componentized way. Welcome to follow us:

* [Google Group](https://groups.google.com/g/js0-site)
* [js0site.bsky.social](https://bsky.app/profile/js0site.bsky.social)

---

<a id="zh"></a>

# self_cmd: 自执行命令构建器

- [概述](#概述)
- [功能特性](#功能特性)
- [使用方法](#使用方法)
- [设计理念](#设计理念)
- [技术栈](#技术栈)
- [项目结构](#项目结构)
- [API 参考](#api-参考)
- [错误处理](#错误处理)

## 概述

`self_cmd` 是 Rust 库，用于创建重新执行当前程序的 Command 实例，保持相同参数和标准输入输出继承。在 Linux 系统上，通过 `LISTEN_FDS` 协议保持文件描述符映射，提供对 systemd 套接字激活的额外支持。

## 功能特性

- **自我复制**: 生成执行当前程序的 Command 对象
- **参数保持**: 自动捕获并转发命令行参数
- **标准输入输出继承**: 维持 stdin、stdout、stderr 连接
- **套接字激活支持**: Linux 特定的文件描述符映射，用于 systemd 集成
- **健壮的错误处理**: 自定义错误类型与正确的错误传播
- **日志集成**: 内置日志支持，用于调试和监控

## 使用方法

添加到 `Cargo.toml`:

```toml
[dependencies]
self_cmd = "0.1.7"
```

```rust
use self_cmd;

fn main() -> self_cmd::Result<()> {
    // 创建并执行重新运行此程序的命令
    // 自动处理标准输入输出继承和 Linux systemd 套接字激活的 FD 映射
    let mut cmd = self_cmd::get()?;
    let status = cmd.status()?;
    println!("子进程退出状态: {status}");
    
    Ok(())
}
```

在 Linux 系统上，库自动处理 systemd 套接字激活，通过检查 `LISTEN_FDS`、从 FD 3 开始复制文件描述符并配置子进程环境。使需要重启但保持监听套接字的服务变得无缝。

## 设计理念

库采用极简主义方法，通过单个公共函数封装自执行的复杂性:

```mermaid
graph TD
    A[程序启动] --> B["self_cmd::get()"]
    B --> C["env::current_exe()"]
    C --> D["env::args().skip(1)"]
    D --> E["Command::new(program)"]
    E --> F["cmd.args(args)"]
    F --> G[配置标准输入输出继承]
    G --> H{Linux?}
    H -->|是| I["fd_mapping()"]
    I --> J[检查 LISTEN_FDS 环境变量]
    J --> K[复制 FD 3+]
    K --> L[创建 FdMapping]
    L --> M[为子进程设置 LISTEN_FDS]
    M --> N[移除 LISTEN_PID]
    N --> O[返回 Command]
    H -->|否| O
    O --> P[执行命令]
```

设计确保:
- **简洁性**: 单函数接口
- **可靠性**: 无恐慌的错误处理
- **灵活性**: 返回 Command 供进一步定制
- **性能**: 惰性求值的最小开销

## 技术栈

- **语言**: Rust 2024 版本
- **核心依赖**: 
  - `log` 用于日志功能
  - `thiserror` 用于结构化错误处理
- **Linux 依赖**:
  - `command-fds` 用于文件描述符映射
  - `libc` 用于底层系统调用
- **错误处理**: 自定义 `Error` 类型与 `thiserror` 集成
- **进程管理**: `std::process::Command` 与 FD 映射扩展
- **环境访问**: `std::env` 获取可执行文件路径和参数

## 项目结构

```
self_cmd/
├── src/
│   ├── lib.rs          # 核心实现和公共 API
│   ├── error.rs        # 错误类型和处理
│   └── fd_mapping.rs   # Linux 特定的 FD 映射（条件编译）
├── tests/
│   └── main.rs         # 集成测试
├── readme/
│   ├── en.md          # 英文文档
│   └── zh.md          # 中文文档
├── Cargo.toml         # 项目配置
└── test.sh           # 测试运行脚本
```

## API 参考

### `get() -> Result<Command>`

创建配置为重新执行当前程序的 Command 实例，完整保持上下文。

**返回值:**
- `Ok(Command)`: 配置好的命令，可直接执行
- `Err(Error)`: 无法确定可执行文件路径或配置 FD 映射时

**行为:**
- 通过 `env::current_exe()` 捕获当前可执行文件路径
- 保留除程序名外的所有命令行参数
- 配置标准输入输出继承 (stdin, stdout, stderr)
- **仅 Linux**: 处理 systemd 套接字激活 FD 映射
- 返回 Command 供执行前进一步定制

**示例:**
```rust
match self_cmd::get() {
    Ok(mut cmd) => {
        // 命令已准备就绪，完整保持上下文
        cmd.status()?;
    }
    Err(e) => {
        eprintln!("创建自执行命令失败: {e}");
    }
}
```

### `fd_mapping() -> Result<Vec<FdMapping>>` (仅 Linux)

**仅在 Linux 上可用** - 为 systemd 套接字激活创建文件描述符映射。

**返回值:**
- `Ok(Vec<FdMapping>)`: 套接字激活的 FD 映射列表
- `Err(Error)`: FD 复制或验证失败时

此函数在 Linux 系统上由 `get()` 自动调用，通常不需要直接使用。

## 错误处理

库使用基于 `thiserror` 构建的自定义 `Error` 类型进行全面的错误处理:

```rust
use self_cmd::{Error, Result};

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0} / IO 错误: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("FD mapping collision: {0} / FD 映射冲突: {0}")]
    FdMappingCollision(#[from] command_fds::FdMappingCollision),
}
```

所有公共函数返回 `Result<T>`，这是 `std::result::Result<T, Error>` 的别名。这提供了:

- **结构化错误**: 不同失败模式的清晰错误类型
- **错误链**: 从底层错误类型的自动转换
- **调试支持**: 带有上下文的丰富错误消息
- **集成**: 与 `?` 操作符和错误处理模式的无缝集成

---

## 关于

本项目为 [js0.site ⋅ 重构互联网计划](https://js0.site) 的开源组件。

我们正在以组件化的方式重新定义互联网的开发范式，欢迎关注：

* [谷歌邮件列表](https://groups.google.com/g/js0-site)
* [js0site.bsky.social](https://bsky.app/profile/js0site.bsky.social)
