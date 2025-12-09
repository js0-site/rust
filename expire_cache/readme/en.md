# expire_cache

Efficient double-buffered expiration cache.

`expire_cache` implements a high-performance cache with expiration based on a "double-buffer" (or generational) strategy. Instead of tracking the expiration time of each individual item, it maintains two buckets (generations) of data. This approach significantly reduces memory overhead and CPU usage for expiration checks, making it ideal for high-throughput scenarios where precise expiration timing is not critical.

## Features

- **High Performance**: Uses a double-buffer strategy for O(1) expiration overhead per item (amortized). No background scanning of all items.
- **Concurrent**: Built on top of `DashMap` for high concurrency.
- **Async Support**: Supports `get_or_init_async` for asynchronous value initialization.
- **Flexible**: Supports both Key-Value cache (`DashMap`) and Set cache (`DashSet`).
- **Simple API**: Easy to use `get`, `insert`, `get_or_init`.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
expire_cache = "0.1"
```

To enable specific features:

```toml
[dependencies]
expire_cache = { version = "0.1", features = ["dashmap", "get_or_init_async"] }
```

Available features:
- `dashmap`: Enable `DashMap` support (default).
- `dashset`: Enable `DashSet` support.
- `get_or_init`: Enable synchronous `get_or_init`.
- `get_or_init_async`: Enable asynchronous `get_or_init_async`.

## Usage

### Basic Usage (Map)

```rust
use std::time::Duration;
use expire_cache::Expire;
use dashmap::DashMap;

#[tokio::main]
async fn main() {
    // Create a cache with a 60-second expiration cycle
    let cache: Expire<DashMap<String, String>> = Expire::new(60);

    cache.insert("key".to_string(), "value".to_string());

    if let Some(val) = cache.get("key") {
        println!("Found: {}", *val);
    }
}
```

### Async Initialization (`get_or_init_async`)

```rust
use expire_cache::{AsyncInit, Expire, Map};
use dashmap::DashMap;

struct DbInit;

impl Map for DbInit {
    type Key = String;
    type Val = String;
    type RefVal<'a> = &'a String;

    fn clear(&self) {}
    fn insert(&self, _key: Self::Key, _val: Self::Val) {}
    fn get<'a>(&'a self, _key: &Self::Key) -> Option<Self::RefVal<'a>> {
        None
    }
}

impl Default for DbInit {
    fn default() -> Self {
        Self
    }
}

impl AsyncInit for DbInit {
    type Key = String;
    type Val = String;
    type Error = anyhow::Error;

    async fn init(_key: &Self::Key) -> Result<Self::Val, Self::Error> {
        // Simulate async work, e.g., database query
        Ok("computed_value".to_string())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cache: Expire<DashMap<String, String>> = Expire::new(60);

    let val = cache.get_or_init_async::<DbInit>("key").await?;

    println!("Value: {}", *val);
    Ok(())
}
```

### Set Usage

```rust
use expire_cache::Expire;
use dashmap::DashSet;

#[tokio::main]
async fn main() {
    let set: Expire<DashSet<String>> = Expire::new(60);

    set.insert("item".to_string(), ());

    if set.get("item").is_some() {
        println!("Item exists");
    }
}
```

## How it Works

1.  **Double Buffering**: The cache maintains two underlying containers (e.g., `DashMap`), let's call them `A` and `B`.
2.  **Active & Passive**: At any time, one is "active" (receiving new inserts) and the other is "passive" (read-only, containing older data).
3.  **Reads**: `get` checks the active container first. If not found, it checks the passive container.
4.  **Writes**: `insert` always writes to the active container.
5.  **Rotation**: Every `expire` seconds, a background task clears the passive container and swaps the roles of `A` and `B`. The previously active container becomes passive (preserving its data for one more cycle), and the cleared container becomes the new active one.

This means an item will live for at least `expire` seconds and at most `2 * expire` seconds.

## License

MulanPSL-2.0