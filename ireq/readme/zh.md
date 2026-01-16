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