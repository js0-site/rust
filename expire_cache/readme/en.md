# expire_cache: High-performance generational cache

`expire_cache` implements an efficient expiration cache using generational collection strategy. Instead of tracking individual item expiration times, it maintains two data buckets (generations), significantly reducing memory overhead and CPU usage for expiration checks.

## Features

- **High Performance**: O(1) amortized expiration overhead per item
- **Concurrent Access**: Built on [papaya](https://docs.rs/papaya) for lock-free thread-safe operations
- **Async Support**: Native async initialization with `get_or_init_async!`
- **Flexible Storage**: Support for both key-value maps and sets
- **Simple API**: Clean interface with `get`, `insert`, and initialization methods

## Installation

```toml
[dependencies]
expire_cache = { version = "0.1.22", features = ["hashmap", "get_or_init_async"] }
```

Available features:

- `hashmap`: Enable HashMap support (papaya::HashMap)
- `hashset`: Enable HashSet support (papaya::HashSet)
- `get_or_init`: Enable synchronous initialization
- `get_or_init_async`: Enable asynchronous initialization macro

## Quick Start

### Basic Usage

```rust
use expire_cache::Expire;
use papaya::HashMap;
use std::time::Duration;

#[tokio::main]
async fn main() {
  let cache: Expire<HashMap<&str, &str>> = Expire::new(60);

  cache.insert("key", "value");

  if let Some(val) = cache.get("key") {
    println!("Found: {val}");
  }

  // Wait for expiration
  tokio::time::sleep(Duration::from_secs(120)).await;
  assert!(cache.get("key").is_none());
}
```

### Async Initialization

```rust
use expire_cache::{Expire, get_or_init_async};
use papaya::HashMap;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
  let cache: Expire<HashMap<String, String>> = Expire::new(60);

  async fn load_data() -> Result<String, std::io::Error> {
    Ok("data_for_user_123".to_string())
  }

  let value: Result<_, std::io::Error> = get_or_init_async!(cache, "user_123", load_data);
  println!("Loaded: {}", value?);

  Ok(())
}
```

### Sync Initialization

```rust
use expire_cache::{Expire, GetOrInit};
use papaya::HashMap;

fn main() -> Result<(), std::io::Error> {
  let cache: Expire<HashMap<String, String>> = Expire::new(60);

  let value = cache.get_or_init("user_123", |key| {
    Ok::<_, std::io::Error>(format!("data_for_{key}"))
  })?;
  println!("Loaded: {value}");

  Ok(())
}
```

### Set Usage

```rust
use expire_cache::Expire;
use papaya::HashSet;

#[tokio::main]
async fn main() {
  let cache: Expire<HashSet<&str>> = Expire::new(60);

  cache.insert("active_session", ());

  if cache.get("active_session").is_some() {
    println!("Session exists");
  }
}
```

## API Reference

### `Expire<T: Map>`

- `new(expire: u64) -> Self`: Create cache with expiration period in seconds
- `get(&self, key) -> Option<RefVal>`: Retrieve value from cache
- `insert(&self, key, val)`: Insert value into cache
- `get_or_init(&self, key, func) -> Result<RefVal, E>`: Sync initialization (requires `get_or_init` feature)

### `get_or_init_async!` macro

- `get_or_init_async!(cache, key, init_fn) -> Result<Val, E>`: Async initialization macro, calls `init_fn()` only on cache miss

#### Why a macro instead of a callback function?

Rust's `impl Trait` return types create distinct opaque types for each call site. When a closure captures references and returns an `impl Future`, the compiler cannot unify the Future's lifetime with the closure parameter's lifetime in generic contexts. This causes "lifetime mismatch" errors.

The macro approach bypasses this by inlining the code at the call site, so the async expression is awaited directly without going through a generic callback, naturally avoiding the lifetime inference issues.

This is a known Rust limitation: [rust-lang/rust#100013](https://github.com/rust-lang/rust/issues/100013)

## Design

### Generational Collection

The cache uses a double-buffer approach with two generations:

1. **Insertion**: New entries always go to the active generation
2. **Lookup**: Check active generation first, then passive generation
3. **Expiration**: Background task periodically clears passive generation and swaps roles
4. **Lifecycle**: Items live between `expire` and `2 * expire` seconds

This approach trades absolute precision for significant throughput improvements and reduced memory fragmentation.

## License

MulanPSL-2.0
