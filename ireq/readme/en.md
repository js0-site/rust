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
