# http_grpc: 将单个HTTP请求转换为多路复用、并发的gRPC式调用

## 目录
- [项目介绍](#项目介绍)
- [设计思路](#设计思路)
- [使用演示](#使用演示)
- [技术堆栈](#技术堆栈)
- [目录结构](#目录结构)
- [相关故事](#相关故事)

### 项目介绍

`http_grpc` 是一个 Rust 包，用于将单个 HTTP 请求转换为多路复用、并发的 gRPC 式调用。它被设计与前端包 [@js0.site/rs2proto](https://www.npmjs.com/package/@js0.site/rs2proto) 协同工作。

此包支持在单个 HTTP 请求中实现多路复用，合并多个 RPC 调用，并利用 HTTP 的 `Transfer-Encoding: chunked` 机制将响应流式传输回客户端。这种方法对于需要并发执行多次数据请求的场景（例如复杂的 Web 应用）非常高效。

### 设计思路

通信协议为效率和并发性而设计。

1.  **客户端请求**：客户端发送 HTTP POST 请求，其 Body 中包含一个或多个二进制帧。
2.  **请求帧结构**：每个帧代表一个远程过程调用，其结构如下：
    `varint(call_id: u32) | varint(func_id: u32) | varint(data_length: u32) | data_payload`
    - `call_id`: 调用的唯一标识，便于客户端将响应与请求对应。
    - `func_id`: 映射到服务端特定函数的标识。
    - `data_length`: `data_payload` 的长度。
    - `data_payload`: 函数的二进制参数。
3.  **服务端处理**：
    - `http_grpc` 函数以流的方式读取 HTTP 请求 Body。
    - 它解析字节流以解码出独立的帧。
    - 对于每个有效的帧，它会生成一个并发的 Tokio 任务，以执行 `Grpc::run` Trait 实现中定义的服务函数。`call_id` 由框架处理，不会传递给 `run` 函数。
4.  **服务端响应**：
    - 服务端函数执行完毕后，其结果被编码为响应帧：
      `varint(call_id: u32) | varint(code: u32) | varint(response_length: u32) | response_payload`
    - `code`: 状态码，类似于 HTTP 状态码。`0` 通常表示成功。
    - 如果 `run` 函数返回 `None`，则发送 `code` 为 0 且 `response_length` 为 0 的响应。
5.  **流式响应**：由于服务端在响应可用时立即发送，客户端通过 HTTP chunked 传输编码以流的形式接收数据，无需等待所有调用完成。

### 使用演示

要使用此包，您必须为您的服务逻辑实现 `Grpc` Trait。`response` 模块提供了辅助函数以简化响应的生成。

```rust
use std::future::Future;
use bytes::Bytes;
use http_grpc::{Grpc, http_grpc, Response};
use xrpc::volo::http::Req;

// 用于处理 RPC 调用的结构体
struct MyService;

// 为您的服务实现 Grpc Trait
impl Grpc for MyService {
  fn run(
    _req: Req, // 包含 HTTP 请求头和扩展
    func_id: u32,
    args: Bytes,
  ) -> impl Future<Output = Option<Response>> + Send {
    async move {
      // 根据 func_id 和 args 处理请求
      // 请求中的 call_id 由框架自动处理
      println!("收到调用 func_id: {}, args: {:?}", func_id, args);

      // 创建响应
      let res = Response {
          code: 0,
          body: "来自服务器的问候!".into(),
      };
      Some(res)
    }
  }
}

// 在您的 Web 服务器中 (例如使用 Axum, Hyper, 或 Salvo):
async fn handle_request(req: volo_http::request::Request<volo_http::body::Body>) {
  // `http_grpc` 函数处理整个生命周期并返回一个响应
  let response = http_grpc::<MyService>(req).await;
  // ... 将响应发送给客户端
}
```

如需完整的可运行示例，请参考 `tests/` 目录中的代码。

### 技术堆栈

- **核心框架**: [Rust](https://www.rust-lang.org/)
- **异步运行时**: [Tokio](https://tokio.rs/)
- **HTTP 抽象**: [Volo-http](https://github.com/cloudwego/volo/tree/main/volo-http) / [Hyper](https://hyper.rs/)
- **编码**: Protocol Buffers (Protobuf) [Varint](https://protobuf.dev/programming-guides/encoding/#varints) 用于帧元数据。

### 目录结构

```
.
├── Cargo.toml      # 包元数据与依赖
├── README.md       # 本文档文件
├── src/
│   ├── lib.rs      # 主模块，导出公共接口
│   ├── error.rs    # 项目的错误类型
│   ├── http_grpc.rs # 请求处理与多路复用的核心逻辑
│   ├── pb.rs       # Protobuf varint 编码/解码逻辑
│   └── response.rs # 响应结构体与序列化辅助函数
└── tests/
    └── main.rs     # 集成测试与使用示例
```

### 相关故事

gRPC，作为本项目所依赖的 RPC 理念，其前身是 Google 一个历史悠久的内部项目 "Stubby"。在十多年的时间里，Stubby 一直是连接 Google 庞大基础设施中成千上万个微服务的核心引擎。然而，Stubby 与 Google 的内部技术栈紧密耦合，不适合公开发布。

随着 HTTP/2 标准的出现，Google 团队看到了一个在现代开放标准之上重建 Stubby 的机会。他们创造了 gRPC，它结合了 HTTP/2 的高效传输和 Protocol Buffers 的强类型接口定义。gRPC 于 2015 年开源，将高性能 RPC 的能力从 Google 内部解放出来，并随后捐赠给云原生计算基金会 (CNCF)，其发展至今依然蓬勃。本项目的灵感正是源于这种利用现代 Web 标准实现稳健高效通信的精神。