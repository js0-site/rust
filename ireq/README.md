[English](#en) | [中文](#zh)

---

<a id="en"></a>

# ireq : Effortless HTTP requests for Rust

- [Introduction](#introduction)
- [Features](#features)
- [Usage](#usage)
- [Design](#design)
- [Tech Stack](#tech-stack)
- [Directory Structure](#directory-structure)
- [API Reference](#api-reference)
- [History](#history)

## Introduction

`ireq` is a streamlined wrapper around the popular `reqwest` library, designed to make HTTP requests in Rust as simple and efficient as possible. It eliminates boilerplate by providing a globally shared, pre-configured client with sensible defaults for timeouts, redirects, and compression. Whether you need raw bytes or a UTF-8 string, `ireq` handles the details so you can focus on your application logic.

## Features

- **Global Static Client**: A lazy-initialized, shared `reqwest::Client` avoids the overhead of creating new clients for every request.
- **Smart Defaults**: Comes configured with a 100s timeout, limited redirects (max 6), and Zstd compression enabled.
- **Auto Proxy**: Automatically detects and uses the `https_proxy` environment variable (requires `proxy` feature).
- **Simplified API**: Direct functions for `get`, `post`, `put`, `delete`, and `patch` that handle URL parsing and response processing.
- **Flexible Output**: Helper functions to get responses as raw `Bytes` or lossy UTF-8 `String`.

## Usage

Add `ireq` to your `Cargo.toml`.

```rust
use ireq::{get, post, getbin};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Simple GET request returning a String
    let html = get("https://httpbin.org/get").await?;
    println!("Response: {}", html);

    // GET request returning raw Bytes
    let data = getbin("https://httpbin.org/image/png").await?;
    println!("Received {} bytes", data.len());

    // POST request with a body
    let response = post("https://httpbin.org/post", "key=value").await?;
    println!("POST Response: {}", response);

    Ok(())
}
```

## Design

The core philosophy of `ireq` is "convention over configuration" for common tasks while retaining the power of `reqwest` when needed.

1.  **Initialization**: The `REQ` static client is initialized on first use via `static_init`. It builds a `reqwest::Client` with a standard configuration.
2.  **Request Flow**:
    - User calls `ireq::get(url)`.
    - `ireq` converts the URL and calls `REQ.get(url)`.
    - The request is passed to the internal `req()` helper.
    - `req()` executes the request, checks for success status codes (200, 204, 308, 307, 206), and returns the body as `Bytes`.
    - `get()` converts the `Bytes` to a `String` (lossy) and returns it.
3.  **Error Handling**: All errors are mapped to `ireq::Error`, simplifying error management.

## Tech Stack

- **[reqwest](https://crates.io/crates/reqwest)**: The industrial-strength HTTP client for Rust.
- **[static_init](https://crates.io/crates/static_init)**: For safe, lazy initialization of the global client.
- **[bytes](https://crates.io/crates/bytes)**: Efficient byte buffer handling.
- **[thiserror](https://crates.io/crates/thiserror)**: Ergonomic error definition.

## Directory Structure

```
.
├── Cargo.toml      # Project configuration and dependencies
├── src
│   ├── lib.rs      # Main library file: exports, static client, helper functions
│   └── error.rs    # Error definitions
└── tests
    └── main.rs     # Integration tests
```

## API Reference

### Data Structures

- **`REQ`**: The global `reqwest::Client` instance. You can use this directly for advanced `reqwest` features not covered by the helper functions.
- **`Error`**: The custom error enum wrapping `reqwest::Error` and handling status errors.
- **`Result<T>`**: Alias for `std::result::Result<T, Error>`.

### Functions

- **`req(req: RequestBuilder) -> Result<Bytes>`**
  Executes a built request, validates the status code, and returns the response body as `Bytes`.

- **`get(url: impl IntoUrl) -> Result<String>`**
  Performs a GET request and returns the response body as a `String` (lossy UTF-8 decoding).

- **`getbin(url: impl IntoUrl) -> Result<Bytes>`**
  Performs a GET request and returns the raw response body as `Bytes`.

- **`post`, `put`, `delete`, `patch`**
  `async fn(url: impl IntoUrl, body: impl Into<Body>) -> Result<String>`
  Perform the respective HTTP method with a request body and return the response as a `String`.

## History

**The First HTTP Request**

In mid-November 1990, at CERN, Tim Berners-Lee wrote the first HTTP client and server. The first version of the protocol, HTTP/0.9, was incredibly simple. It had only one method, `GET`, and did not support headers or status codes. The client simply sent `GET /path`, and the server streamed back the HTML document, closing the connection immediately after. There were no content types, no version numbers, and no error codes—if something went wrong, you just got a human-readable error message in the HTML or a closed connection. From these humble beginnings, we now have the complex, feature-rich web of today, powered by libraries like `reqwest` and simplified by tools like `ireq`.

---

## About

This project is an open-source component of [js0.site ⋅ Refactoring the Internet Plan](https://js0.site).

We are redefining the development paradigm of the Internet in a componentized way. Welcome to follow us:

- [Google Group](https://groups.google.com/g/js0-site)
- [js0site.bsky.social](https://bsky.app/profile/js0site.bsky.social)

---

<a id="zh"></a>

# ireq : Rust 极简 HTTP 请求库

- [简介](#简介)
- [特性](#特性)
- [使用演示](#使用演示)
- [设计思路](#设计思路)
- [技术堆栈](#技术堆栈)
- [目录结构](#目录结构)
- [API 参考](#api-参考)
- [历史小故事](#历史小故事)

## 简介

`ireq` 是对业界标杆 `reqwest` 库的轻量级封装，旨在极致简化 Rust 中的 HTTP 请求操作。通过提供预配置的全局共享客户端、智能默认设置以及自动代理检测，`ireq` 消除繁琐的样板代码。无论是获取原始字节流还是 UTF-8 字符串，`ireq` 都能让开发者专注于核心业务逻辑。

## 特性

- **全局静态客户端**：采用懒加载机制初始化的共享 `reqwest::Client`，避免重复创建客户端带来的开销。
- **智能默认配置**：内置 100秒超时、限制 6次重定向以及 Zstd 压缩支持。
- **自动代理检测**：自动识别并使用 `https_proxy` 环境变量（需开启 `proxy` 特性）。
- **极简 API**：提供 `get`、`post`、`put`、`delete`、`patch` 等直观函数，自动处理 URL 解析及响应。
- **灵活输出**：支持获取原始 `Bytes` 或有损转换的 UTF-8 `String`。

## 使用演示

在 `Cargo.toml` 中添加 `ireq` 依赖。

```rust
use ireq::{get, post, getbin};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 简单的 GET 请求，返回 String
    let html = get("https://httpbin.org/get").await?;
    println!("响应内容: {}", html);

    // GET 请求，返回原始 Bytes
    let data = getbin("https://httpbin.org/image/png").await?;
    println!("接收到 {} 字节", data.len());

    // 带有 Body 的 POST 请求
    let response = post("https://httpbin.org/post", "key=value").await?;
    println!("POST 响应: {}", response);

    Ok(())
}
```

## 设计思路

`ireq` 的核心理念是“约定优于配置”，在保留 `reqwest` 强大功能的同时，针对通用场景进行优化。

1.  **初始化流程**：利用 `static_init` 实现 `REQ` 静态客户端的首次使用即初始化，构建包含标准配置的 `reqwest::Client`。
2.  **调用流程**：
    - 用户调用 `ireq::get(url)`。
    - `ireq` 解析 URL 并调用 `REQ.get(url)`。
    - 请求传递至内部 `req()` 辅助函数。
    - `req()` 执行请求，校验状态码（200, 204, 308, 307, 206），并返回 `Bytes`。
    - `get()` 将 `Bytes` 转换为 `String`（有损）并返回。
3.  **错误处理**：所有错误统一映射为 `ireq::Error`，简化错误管理。

## 技术堆栈

- **[reqwest](https://crates.io/crates/reqwest)**：Rust 生态中工业级的 HTTP 客户端。
- **[static_init](https://crates.io/crates/static_init)**：用于安全、懒加载的全局变量初始化。
- **[bytes](https://crates.io/crates/bytes)**：高效的字节缓冲区处理。
- **[thiserror](https://crates.io/crates/thiserror)**：优雅的错误定义库。

## 目录结构

```
.
├── Cargo.toml      # 项目配置及依赖
├── src
│   ├── lib.rs      # 核心库文件：导出接口、静态客户端、辅助函数
│   └── error.rs    # 错误定义
└── tests
    └── main.rs     # 集成测试
```

## API 参考

### 数据结构

- **`REQ`**：全局 `reqwest::Client` 实例。如需使用 `reqwest` 的高级功能，可直接调用此实例。
- **`Error`**：自定义错误枚举，封装 `reqwest::Error` 并处理状态码错误。
- **`Result<T>`**：`std::result::Result<T, Error>` 的别名。

### 函数接口

- **`req(req: RequestBuilder) -> Result<Bytes>`**
  执行构建好的请求，校验状态码，并以 `Bytes` 形式返回响应体。

- **`get(url: impl IntoUrl) -> Result<String>`**
  执行 GET 请求，返回 `String` 格式的响应体（有损 UTF-8 解码）。

- **`getbin(url: impl IntoUrl) -> Result<Bytes>`**
  执行 GET 请求，返回原始 `Bytes` 格式的响应体。

- **`post`, `put`, `delete`, `patch`**
  `async fn(url: impl IntoUrl, body: impl Into<Body>) -> Result<String>`
  执行相应的 HTTP 方法并附带请求体，返回 `String` 格式的响应。

## 历史小故事

**第一次 HTTP 请求**

1990 年 11 月中旬，在欧洲核子研究组织（CERN），Tim Berners-Lee 编写了世界上第一个 HTTP 客户端和服务器。最初的 HTTP/0.9 协议极其简陋，仅支持一种方法：`GET`。它不支持 HTTP 头，也没有状态码。客户端只需发送 `GET /path`，服务器便会流式传输 HTML 文档，并在传输结束后立即关闭连接。没有内容类型，没有版本号，也没有错误代码——如果出错，你只能在 HTML 中看到一段可读的错误信息，或者直接断开连接。正是从这简陋的起点出发，经过无数次迭代，才诞生了如今功能丰富、结构复杂的万维网，以及像 `reqwest` 这样强大的工具和 `ireq` 这样便捷的封装库。

---

## 关于

本项目为 [js0.site ⋅ 重构互联网计划](https://js0.site) 的开源组件。

我们正在以组件化的方式重新定义互联网的开发范式，欢迎关注：

- [谷歌邮件列表](https://groups.google.com/g/js0-site)
- [js0site.bsky.social](https://bsky.app/profile/js0site.bsky.social)
