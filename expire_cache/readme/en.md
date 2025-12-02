# expire_cache : Efficient double-buffered expiration cache

## Table of Contents
- Introduction
- Features
- Usage
- Design
- Tech Stack
- Directory Structure
- API Reference
- Historical Context

## Introduction
`expire_cache` implements high-performance, concurrent cache with automatic expiration. It utilizes double-buffering strategy to manage object lifecycles, ensuring efficient memory usage and zero-cost expiration checks during access.

## Features
- **High Performance**: Uses `dashmap` or `dashset` for concurrent access.
- **Low Overhead**: Double-buffering eliminates need for per-item timestamp checks.
- **Memory Efficient**: Single allocation for internal state; bulk deallocation of expired items.
- **Thread Safe**: Fully thread-safe using atomic operations and `unsafe` optimizations for raw pointer access.
- **Async Support**: Background timer for expiration runs on Tokio runtime.

## Usage
Add to `Cargo.toml`:
```toml
[dependencies]
expire_cache = "0.1"
```

Example:
```rust
use std::time::Duration;
use expire_cache::Expire;
use dashmap::DashMap;

#[tokio::main]
async fn main() {
    // Initialize cache with 5 seconds expiration
    let cache: Expire<DashMap<String, String>> = Expire::new(5);

    cache.insert(&"key".to_string(), "value".to_string());

    if let Some(val) = cache.get(&"key".to_string()) {
        println!("Found: {}", val);
    }

    tokio::time::sleep(Duration::from_secs(6)).await;

    // Item expired
    assert!(cache.get(&"key".to_string()).is_none());
}
```

## Design
Core mechanism relies on two underlying maps (buffers) and an atomic index.

1.  **Structure**: `Inner` struct holds `[T; 2]` (two maps) and `AtomicUsize` (index).
2.  **Insertion**: Always writes to active buffer determined by atomic index.
3.  **Retrieval**: Checks active buffer first; if missing, checks inactive buffer. This ensures items remain accessible for at least one full expiration cycle.
4.  **Expiration**: Background Tokio task wakes up every `expire` seconds. It toggles atomic index, effectively swapping active/inactive buffers, and clears the new active buffer (which holds oldest data).

## Tech Stack
- **Rust**: Core language.
- **Tokio**: Async runtime for background expiration task.
- **DashMap**: High-performance concurrent hash map.
- **Atomic/Unsafe**: For lock-free state management and optimized memory layout.

## Directory Structure
```
.
├── Cargo.toml
├── readme
│   ├── en.md
│   └── zh.md
├── src
│   ├── lib.rs      # Core logic and Expire struct
│   ├── map.rs      # Map trait implementation for DashMap
│   └── set.rs      # Map trait implementation for DashSet
└── tests
    └── main.rs     # Integration tests
```

## API Reference

### `Expire<T>`
Main struct managing cache state. `T` must implement `Map` trait.

#### `new(expire: u64) -> Self`
Creates new cache instance. `expire` specifies rotation interval in seconds.

#### `insert(&self, key: T::Key, val: T::Val)`
Inserts key-value pair into active cache.

#### `get(&self, key: impl Borrow<T::Key>) -> Option<T::RefVal<'_>>`
Retrieves value. Checks active cache first, then inactive cache. Returns a reference guard.

### `Map` Trait
Abstraction over underlying storage (e.g., `DashMap`, `DashSet`).

## Historical Context
Concept of "generational caching" or "cache rotation" mirrors strategies found in hardware design and garbage collection. Early mainframe systems like IBM System/360 introduced caching to bridge CPU-memory speed gaps. Over time, strategies evolved from simple LRU (Least Recently Used) to more complex generational approaches.

In hardware, "cache decay" reduces power consumption by turning off cache lines that haven't been accessed for a "generation". Similarly, `expire_cache` treats time intervals as generations. By bulk-discarding entire generations of data, it avoids overhead of tracking individual item timestamps—a technique reminiscent of generational garbage collectors which assume "young" objects die young and can be collected in bulk. This approach trades absolute precision (exact expiration time) for significant throughput gains and reduced memory fragmentation.