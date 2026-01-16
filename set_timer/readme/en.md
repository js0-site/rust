# set_timer : Robust Interval Execution for Rust

> A robust interval execution library for Rust, supporting both async and sync closures with Tokio integration.

## Features

- **Dual Mode Support**: Seamlessly handles both synchronous and asynchronous closures.
- **Tokio Integration**: Built on top of the powerful `tokio` runtime for efficient task scheduling.
- **Easy Cancellation**: Returns a `JoinHandle` allowing for immediate and clean task abortion.
- **Familiar API**: Designed to mirror the simplicity of JavaScript's `setInterval` while leveraging Rust's safety and performance.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
set_timer = "0.1.0"
```

### Synchronous Interval

For simple, blocking operations or quick computations:

```rust
use std::time::Duration;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use set_timer::set_timer;

#[tokio::main]
async fn main() {
    let count = Arc::new(AtomicUsize::new(0));
    let count_clone = count.clone();

    // Execute every 100ms
    let handle = set_timer(
        move || {
            count_clone.fetch_add(1, Ordering::SeqCst);
            println!("Tick");
        },
        Duration::from_millis(100),
    );

    // Let it run for a while
    tokio::time::sleep(Duration::from_millis(350)).await;

    // Stop the interval
    handle.abort();
}
```

### Asynchronous Interval

For operations involving I/O, network requests, or other async tasks:

```rust
use std::time::Duration;
use set_timer::set_timer_async;

#[tokio::main]
async fn main() {
    let handle = set_timer_async(
        || async {
            // Perform async work here
            println!("Async Tick");
        },
        Duration::from_millis(100),
    );

    tokio::time::sleep(Duration::from_millis(350)).await;
    handle.abort();
}
```

## Design

The library leverages Tokio's lightweight task spawning system.

1.  **Task Spawning**: When `set_timer` or `set_timer_async` is called, a new Tokio task is spawned using `tokio::spawn`.
2.  **Loop Execution**: Inside the task, an infinite `loop` is established.
3.  **Execution & Wait**:
    - The provided closure is executed (awaited if async).
    - The loop then pauses execution using `tokio::time::sleep` for the specified `period`.
4.  **Resource Management**: The returned `JoinHandle` serves as a control mechanism. Dropping the handle doesn't stop the task, but calling `.abort()` on it terminates the loop immediately, ensuring resources are freed.

## Tech Stack

- **Language**: Rust (Edition 2024)
- **Runtime**: Tokio (Async runtime, Time utilities)
- **Testing**: Tokio Test, Aok

## Directory Structure

```
.
├── Cargo.toml          # Project configuration and dependencies
├── src/
│   └── lib.rs          # Core library logic (set_timer, set_timer_async)
└── tests/
    └── main.rs         # Integration tests and usage examples
```

## API Reference

### `set_timer`

```rust
pub fn set_timer<F>(func: F, period: Duration) -> JoinHandle<()>
where
    F: Fn() + Send + Sync + 'static
```

Executes a synchronous closure `func` repeatedly with a fixed time delay `period` between each call.

### `set_timer_async`

```rust
pub fn set_timer_async<F, Fut>(func: F, period: Duration) -> JoinHandle<()>
where
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send
```

Executes an asynchronous closure `func` repeatedly. The `period` delay occurs after the future returned by `func` completes.

## History

The concept of `setInterval` originates from the early days of web development. While often associated with JavaScript, it is not actually part of the core ECMAScript specification. Instead, it was introduced as part of the "host environment" API provided by web browsers (the `Window` interface) and later adopted by Node.js.

It became a cornerstone of dynamic web pages, enabling everything from simple digital clocks to complex polling mechanisms and animations. This crate aims to bring that same essential utility to the Rust ecosystem, adapted for the modern era of asynchronous systems programming.