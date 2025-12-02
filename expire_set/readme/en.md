# expire_set : High-performance concurrent expiration set

A high-performance, concurrent set with automatic item expiration, implemented using `unsafe` raw pointers for maximum efficiency.

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

Unlike traditional TTL caches that store a timestamp for *every single item*, `expire_set` uses a double-buffering strategy. This approach eliminates the memory overhead of per-item timestamps and the CPU overhead of checking them, making it extremely memory-efficient and fast.

## Features

- **Memory Efficient**: Does **not** store expiration timestamps for individual items. Saves significant memory when caching millions of small items (like IP addresses or URLs).
- **Ideal for Short-Lived Cache**: Perfect for use cases like "expire after 1 minute," such as 404 flooding protection or deduplication buffers.
- **High Performance**: Uses `unsafe` raw pointers and `AtomicUsize` to avoid `Arc` reference counting overhead.
- **Concurrency**: Built on `DashSet` for thread-safe, concurrent access.
- **Automatic Bulk Expiration**: Background timer rotates buffers to expire old items in bulk, rather than scanning for expired items one by one.
- **Zero Overhead Sharing**: State is shared between the timer and the main struct using raw pointers.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
expire_set = "0.1.0"
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

The core design is based on a **Double Buffering** (or Rotating Cache) mechanism:

1.  **Two Buffers**: The struct holds two `DashSet` instances.
2.  **Atomic Index**: An `AtomicUsize` indicates the "current" active buffer (0 or 1).
3.  **Insertion**: New items are always inserted into the `current` buffer.
4.  **Querying**: `contains` checks *both* buffers to ensure items are valid until they are fully cleared.
5.  **Rotation**: A background Tokio task sleeps for the `expire` duration. Upon waking, it flips the index (0 -> 1 or 1 -> 0) and clears the *new* current buffer (which holds the oldest data).

This approach avoids checking timestamps for every item. Instead, items expire in bulk when their buffer is cleared.

### Unsafe Optimization

To satisfy strict performance requirements, `Arc` is bypassed in favor of `unsafe` raw pointers (`*const`) and `Box::leak`.
-   Data is leaked to the heap with `'static` lifetime.
-   Pointers are wrapped in a `SendPtr` struct to allow passing them to the background task.
-   Memory is manually reclaimed in `Drop`.

## API Documentation

### `ExpireSet<K>`

The main struct. `K` must implement `Hash + Eq + Clone + Send + Sync + 'static`.

#### `fn new(expire: u64) -> Self`
Creates a new `ExpireSet`.
-   `expire`: The duration in seconds before the buffer rotates. Items live for roughly `expire` to `2 * expire` seconds.

#### `fn insert(&self, key: K)`
Inserts a key into the current active set.

#### `fn contains(&self, key: &K) -> bool`
Checks if the key exists in either the current or the previous set.

## Technology Stack

-   **Rust**: Core language.
-   **Tokio**: Async runtime for the background timer task.
-   **DashMap**: Concurrent associative array for storage.
-   **Atomic**: Standard library atomics for synchronization.

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
    └── main.rs         # Integration tests
```

## Historical Trivia

**The Origin of Double Buffering**

The "rotating cache" technique used in this project is analogous to **Double Buffering** in computer graphics.

Double buffering originated in the late 1960s and became standard in the 1980s with systems like the **Amiga**. In graphics, it involves drawing to a hidden "back buffer" while displaying the "front buffer," then swapping them instantly to prevent screen tearing.

Similarly, `expire_set` writes to a "current" buffer while keeping the "previous" buffer available for reads. When the timer fires, it "swaps" the buffers (by changing the index) and clears the old one, ensuring a smooth transition and efficient bulk expiration, much like the artifact-free rendering in early graphics hardware.