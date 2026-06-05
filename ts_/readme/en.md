# ts\_ : High-performance, lightweight time measurement for Rust

`ts_` is a simplified wrapper around the `coarsetime` crate, designed to provide extremely fast and easy-to-use time measurement utilities. It focuses on speed and API stability, making it ideal for performance-critical applications where high-precision (but not necessarily atomic-clock level) timestamps are needed without the overhead of standard library calls.

## Table of Contents

- [Features](#features)
- [Usage](#usage)
- [Design Philosophy](#design-philosophy)
- [Tech Stack](#tech-stack)
- [Directory Structure](#directory-structure)
- [API Reference](#api-reference)
- [History: The Quest for Speed](#history-the-quest-for-speed)

## Features

- **High Performance**: Leverages `CLOCK_MONOTONIC_COARSE` on Linux systems for minimal overhead.
- **Simplicity**: Exposes a straightforward API with `nano()`, `milli()`, and `sec()` functions.
- **Stability**: Consistent behavior across different platforms, avoiding runtime panics common in some standard library implementations.
- **Lightweight**: Minimal dependencies and optimized for speed.
- **Flexible Precision**: Choose between nanoseconds, milliseconds, or seconds based on your needs.

## Usage

Add `ts_` to your `Cargo.toml`:

```toml
[dependencies]
ts_ = "0.1.2"
```

Enable the features you need:

```toml
[dependencies]
ts_ = { version = "0.1.2", features = ["nano", "milli", "sec"] }
```

### Example

Here is a simple example demonstrating how to retrieve the current time in different precisions:

```rust
use aok::{OK, Void};
use log::info;

#[test]
fn test() -> Void {
  // Get current time in nanoseconds (requires "nano" feature)
  info!("{}", ts_::nano());

  // Get current time in milliseconds (requires "milli" feature)
  info!("{}", ts_::milli());

  // Get current time in seconds (requires "sec" feature)
  info!("{}", ts_::sec());

  OK
}
```

## Design Philosophy

The core design philosophy of `ts_` is **"Speed over Accuracy"**.

1.  **Underlying Mechanism**: It wraps `coarsetime`, which uses `CLOCK_MONOTONIC_COARSE` on Linux. This avoids expensive system calls by reading the time directly from a memory page updated by the kernel (vDSO).
2.  **Simplified API**: Instead of dealing with complex `Duration` or `Instant` objects for simple timestamp needs, `ts_` provides direct access to `u64` values representing time.
3.  **Feature Flags**: Users can opt-in for `nano` or `sec` precision features to keep the compilation minimal.

## Tech Stack

- **Language**: Rust
- **Core Dependency**: `coarsetime`
- **Repository**: https://github.com/js0-site/rust.git
- **License**: MulanPSL-2.0

## Directory Structure

```
.
├── src/
│   └── lib.rs      # Core implementation exporting nano() and sec()
├── tests/
│   └── main.rs     # Integration tests and usage examples
├── Cargo.toml      # Project configuration
└── readme/         # Documentation
```

## API Reference

### `nano()`

```rust
pub fn nano() -> u64
```

Returns the current time in nanoseconds since the epoch. Requires the `nano` feature.

### `milli()`

```rust
pub fn milli() -> u64
```

Returns the current time in milliseconds since the epoch. Requires the `milli` feature.

### `sec()`

```rust
pub fn sec() -> u64
```

Returns the current time in seconds since the epoch. Requires the `sec` feature.

## History: The Quest for Speed

In the early days of Linux, getting the time was always a system call. A system call involves switching from user mode to kernel mode, which is a relatively expensive operation for the CPU. For high-performance applications that needed to check the time thousands or millions of times per second (like high-frequency trading or high-throughput servers), this overhead was significant.

To solve this, the Linux kernel introduced the **vDSO (Virtual Dynamic Shared Object)** mechanism. This allows the kernel to map a small area of memory into the user space of every process. This memory contains frequently used data, such as the current time.

`CLOCK_MONOTONIC_COARSE` was born from this innovation. It reads the time directly from this shared memory without triggering a context switch. While it might be slightly less precise than a full system call (updating only on timer ticks), it is orders of magnitude faster. `ts_` (via `coarsetime`) harnesses this power to give your Rust applications blazing fast time measurements.
