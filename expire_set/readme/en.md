# expire_set : High-performance concurrent expiration set

A high-performance, concurrent set with automatic item expiration, built on top of `expire_cache` using unsafe raw pointers for maximum performance.

## Table of Contents

- [Introduction](#introduction)
- [Features](#features)
- [Usage](#usage)
- [Design Philosophy](#design-philosophy)
- [API Documentation](#api-documentation)
- [Technology Stack](#technology-stack)
- [Directory Structure](#directory-structure)
- [Historical Trivia](#historical-trivia)

## Introduction

`expire_set` is a specialized Rust library designed for high-throughput scenarios where items need to expire after a short duration, such as **caching 404 request paths** to prevent DoS attacks.

It is a wrapper around `expire_cache`, providing a simplified `Set` interface while leveraging the efficient double-buffering expiration strategy of the underlying library. The implementation uses unsafe raw pointers for optimal performance in concurrent environments.

## Features

- **Memory Efficient**: Does **not** store expiration timestamps for individual items.
- **Ideal for Short-Lived Cache**: Perfect for use cases like "expire after 1 minute," such as 404 flooding protection or deduplication buffers.
- **High Performance**: Leverages `expire_cache`'s lock-free and double-buffering mechanisms with unsafe raw pointers.
- **Concurrency**: Thread-safe concurrent access.
- **Automatic Bulk Expiration**: Background timer rotates buffers to expire old items in bulk.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
expire_set = "0.1.7"
tokio = { version = "1", features = ["time", "rt"] }
```

Example usage:

```rust
use expire_set::ExpireSet;
use std::time::Duration;

#[tokio::main]
async fn main() {
    // Create a set where items expire every 10 seconds
    let set = ExpireSet::<String>::new(10);

    // Insert items
    set.insert("key1".to_string());

    // Check existence
    if set.contains(&"key1".to_string()) {
        println!("Key exists!");
    }

    // Wait for expiration
    tokio::time::sleep(Duration::from_secs(25)).await;

    // Item should be gone
    assert!(!set.contains(&"key1".to_string()));
}
```

## Design Philosophy

The core design relies on the **Double Buffering** mechanism provided by `expire_cache`:

1.  **Underlying Storage**: Uses `expire_cache::Expire<HashSet<K>>` internally (powered by Papaya).
2.  **Simplified Interface**: Exposes `insert` and `contains` methods tailored for Set operations.
3.  **Efficient Expiration**: Inherits the bulk expiration strategy where items are cleared in generations rather than individually.

## API Documentation

### `ExpireSet<K>`

The main struct. `K` must implement `Hash + Eq + Send + Sync + 'static`.

#### `fn new(expire: u64) -> Self`

Creates a new `ExpireSet`.

- `expire`: The duration in seconds before the buffer rotates. Items live for roughly `expire` to `2 * expire` seconds.

#### `fn insert(&self, key: K)`

Inserts a key into the set.

#### `fn contains<Q>(&self, key: &Q) -> bool`

Checks if the key exists in the set.

- `Q`: The query type that can be borrowed as `K`
- Returns: `true` if the key exists, `false` otherwise

## Technology Stack

- **Rust**: Core language.
- **expire_cache**: Underlying expiration logic.
- **Tokio**: Async runtime.
- **Papaya**: Fast and ergonomic concurrent hash-table (via `expire_cache`).

## Directory Structure

```
.
├── Cargo.toml          # Project configuration
├── readme/             # Documentation
│   ├── en.md           # English README
│   └── zh.md           # Chinese README
├── src/
│   └── lib.rs          # Source code (ExpireSet implementation)
└── tests/
│   └── main.rs         # Integration tests
```

## Historical Trivia

**The Origin of Double Buffering**

The "rotating cache" technique used in this project is analogous to **Double Buffering** in computer graphics.

Double buffering originated in the late 1960s and became standard in the 1980s with systems like the **Amiga**. In graphics, it involves drawing to a hidden "back buffer" while displaying the "front buffer," then swapping them instantly to prevent screen tearing.

Similarly, `expire_set` (via `expire_cache`) writes to a "current" buffer while keeping the "previous" buffer available for reads. When the timer fires, it "swaps" the buffers and clears the old one, ensuring a smooth transition and efficient bulk expiration.
