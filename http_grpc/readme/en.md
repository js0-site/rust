# http_grpc: Convert a single HTTP request into multiplexed, concurrent gRPC-like calls

## Table of Contents
- [Introduction](#introduction)
- [How It Works](#how-it-works)
- [Usage Example](#usage-example)
- [Technology Stack](#technology-stack)
- [Directory Structure](#directory-structure)
- [A Little Story](#a-little-story)

### Introduction

`http_grpc` is a Rust crate that converts a single HTTP request into multiplexed, concurrent gRPC-like calls. It is designed to work with the frontend package [@js0.site/rs2proto](https://www.npmjs.com/package/@js0.site/rs2proto).

This crate enables multiplexing multiple RPC calls within one HTTP request and streams responses back to the client using HTTP's `Transfer-Encoding: chunked` mechanism. This approach is efficient for scenarios requiring multiple, concurrent data fetches, such as in complex web applications.

### How It Works

The communication protocol is designed for efficiency and concurrency.

1.  **Client Request**: The client sends an HTTP POST request. The body of this request contains one or more binary frames.
2.  **Request Frame**: Each frame represents a single remote procedure call and is structured as follows:
    `varint(call_id: u32) | varint(func_id: u32) | varint(data_length: u32) | data_payload`
    - `call_id`: A unique identifier for the call, allowing the client to match responses to requests.
    - `func_id`: An identifier that maps to a specific function on the server.
    - `data_length`: The length of the `data_payload`.
    - `data_payload`: The binary arguments for the function.
3.  **Server Processing**:
    - The `http_grpc` function reads the HTTP request body as a stream.
    - It parses the stream to decode individual frames.
    - For each valid frame, it spawns a concurrent Tokio task to execute the corresponding server function, as defined by the `Grpc::run` trait implementation. The `call_id` is handled by the framework and is not passed to the `run` function.
4.  **Server Response**:
    - After a server function completes, its result is encoded into a response frame:
      `varint(call_id: u32) | varint(code: u32) | varint(response_length: u32) | response_payload`
    - `code`: A status code, similar to HTTP status codes. `0` typically indicates success.
    - If the `run` function returns `None`, a response with `code` 0 and `response_length` 0 is sent.
5.  **Streaming Response**: Because the server sends response frames as they become available, the client receives data in a stream via HTTP chunked transfer encoding, without waiting for all calls to complete.

### Usage Example

To use this crate, you must implement the `Grpc` trait for your service logic. The `response` module provides helpers to facilitate response generation.

```rust
use std::future::Future;
use bytes::Bytes;
use http_grpc::{Grpc, http_grpc, Response};
use xrpc::volo::http::Req;

// Your struct that will handle the RPC calls.
struct MyService;

// Implement the Grpc trait for your service.
impl Grpc for MyService {
  fn run(
    _req: Req, // Contains HTTP request headers and extensions.
    func_id: u32,
    args: Bytes,
  ) -> impl Future<Output = Option<Response>> + Send {
    async move {
      // Process the request based on func_id and args.
      // The call_id from the request is handled automatically by the framework.
      println!("Received call with func_id: {}, args: {:?}", func_id, args);

      // Create a response.
      let res = Response {
          code: 0,
          body: "Hello from server!".into(),
      };
      Some(res)
    }
  }
}

// In your web server (e.g., using Axum, Hyper, or Salvo):
async fn handle_request(req: volo_http::request::Request<volo_http::body::Body>) {
  // The `http_grpc` function handles the entire lifecycle and returns a response.
  let response = http_grpc::<MyService>(req).await;
  // ... send the response to the client.
}
```

For a complete, runnable example, please refer to the code in the `tests/` directory.

### Technology Stack

- **Core Framework**: [Rust](https://www.rust-lang.org/)
- **Asynchronous Runtime**: [Tokio](https://tokio.rs/)
- **HTTP Abstractions**: [Volo-http](https://github.com/cloudwego/volo/tree/main/volo-http) / [Hyper](https://hyper.rs/)
- **Encoding**: Protocol Buffers (Protobuf) [Varint](https://protobuf.dev/programming-guides/encoding/#varints) for framing metadata.

### Directory Structure

```
.
├── Cargo.toml      # Package metadata and dependencies
├── README.md       # This documentation file
├── src/
│   ├── lib.rs      # Main module, exports public interfaces
│   ├── error.rs    # Error types for the crate
│   ├── http_grpc.rs # Core logic for request handling and multiplexing
│   ├── pb.rs       # Protobuf varint encoding/decoding logic
│   └── response.rs # Response struct and serialization helpers
└── tests/
    └── main.rs     # Integration tests and usage examples
```

### A Little Story

gRPC, the underlying RPC philosophy, was born out of a long-standing internal Google project named "Stubby." For over a decade, Stubby was the workhorse connecting the thousands of microservices within Google's massive infrastructure. However, Stubby was tightly coupled to Google's internal technologies and wasn't suitable for public release.

With the advent of the HTTP/2 standard, the Google team saw an opportunity to rebuild Stubby on modern, open standards. They created gRPC, combining the efficiency of HTTP/2 for transport and the strong typing of Protocol Buffers for interface definition. Open-sourced in 2015, gRPC extended the power of high-performance RPC beyond Google, and it was later donated to the Cloud Native Computing Foundation (CNCF), where its development continues to thrive. This project is inspired by that spirit of leveraging modern web standards for robust and efficient communication.