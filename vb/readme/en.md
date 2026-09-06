# vb

**The fastest VByte encoding library in the Rust ecosystem.**

Encode at **430M integers/sec**, decode at **415M integers/sec** — 2.4x faster encoding and 1.2x faster decoding than alternatives.

![VByte Encoding Benchmark](https://raw.githubusercontent.com/js0-site/rust/refs/heads/main/vb/svg/en.svg)

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Usage](#usage)
  - [Basic Encoding](#basic-encoding)
  - [Differential Encoding](#differential-encoding)
- [API Reference](#api-reference)
- [Performance](#performance)
- [Design](#design)

## Features

- **Blazing Fast**: Hand-optimized with loop unrolling, bounds check elimination, and CLZ instructions
- **Variable Byte Encoding**: Compresses `u64` integers using 1-10 bytes based on magnitude
- **Differential Encoding**: Optimizes strictly increasing sequences (requires `diff` feature)
- **Zero-Copy Decoding**: Decode directly from byte slices with offset tracking
- **Minimal Dependencies**: Only `thiserror` for error handling

## Installation

```toml
[dependencies]
vb = "0.2"

# With differential encoding support
vb = { version = "0.2", features = ["diff"] }
```

## Usage

### Basic Encoding

```rust
use vb::{e_li, d_li};

let numbers = vec![0, 127, 128, 16383, 16384, 2097151];

// Encode
let encoded = e_li(numbers.iter().cloned());
println!("Compressed to {} bytes", encoded.len());

// Decode
let decoded = d_li(&encoded).unwrap();
assert_eq!(numbers, decoded);
```

### Differential Encoding

Ideal for sorted sequences like timestamps, IDs, or offsets.

```rust
use vb::{e_diff, d_diff};

let timestamps = vec![1000000, 1000005, 1000010, 1000042];

// Stores only deltas: [1000000, 5, 5, 32]
let encoded = e_diff(&timestamps);

let decoded = d_diff(&encoded).unwrap();
assert_eq!(timestamps, decoded);
```

## API Reference

| Function        | Description                                           |
| --------------- | ----------------------------------------------------- |
| `e(value, buf)` | Encode single `u64`, append to buffer                 |
| `d(bytes)`      | Decode single `u64`, return `(value, bytes_consumed)` |
| `e_li(iter)`    | Encode iterator of `u64` to `Vec<u8>`                 |
| `d_li(bytes)`   | Decode bytes to `Vec<u64>`                            |
| `e_diff(slice)` | Encode increasing sequence with delta compression     |
| `d_diff(bytes)` | Decode delta-compressed sequence                      |

## Performance

Benchmarked with 10,000 integers (60% small, 30% medium, 10% large):

| Library          | Encode (M/s) | Decode (M/s) |
| ---------------- | ------------ | ------------ |
| **vb**           | **430**      | **415**      |
| leb128           | 289          | 213          |
| integer-encoding | 176          | 349          |

Run benchmarks yourself:

```bash
./bench.sh
```

## Design

VByte uses 7 bits per byte for data, with the MSB as continuation flag:

- `MSB = 0`: Final byte
- `MSB = 1`: More bytes follow

Key optimizations:

- **Fast path**: Single-byte values (< 128) skip all loops
- **Loop unrolling**: 2-5 byte cases fully unrolled
- **Bounds elimination**: Unsafe pointer arithmetic when ≥10 bytes available
- **CLZ instruction**: `leading_zeros()` calculates byte count in one CPU cycle
