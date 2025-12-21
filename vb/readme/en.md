# vb : Efficient compression for integer sequences

## Table of Contents
- [Introduction](#introduction)
- [Features](#features)
- [Usage](#usage)
  - [Basic Encoding](#basic-encoding)
  - [Differential Encoding](#differential-encoding)
- [API Reference](#api-reference)
- [Design](#design)
- [Tech Stack](#tech-stack)
- [Directory Structure](#directory-structure)
- [History](#history)

## Introduction
`vb` is a lightweight Rust library for **Variable Byte (VByte)** encoding. It provides an efficient way to compress `u64` integers and lists of integers. By using fewer bytes for smaller numbers, it significantly reduces storage requirements for integer-heavy data.

Additionally, the library supports **Differential Encoding (Delta Encoding)**, which is particularly effective for compressing strictly increasing sequences (e.g., sorted IDs, timestamps) by storing only the differences between consecutive values.

## Features
- **Variable Byte Encoding**: Compresses `u64` integers into a variable-length byte sequence.
- **Differential Encoding**: Optimizes storage for strictly increasing sequences (requires `diff` feature).
- **Zero-Copy Decoding**: Efficient decoding directly from byte slices.
- **Simple API**: Easy-to-use functions for single values and lists.
- **Error Handling**: Robust error reporting using `thiserror`.

## Usage

Add `vb` to your `Cargo.toml`:

```toml
[dependencies]
vb = "0.1.8"
# For differential encoding support
# vb = { version = "0.1.8", features = ["diff"] }
```

### Basic Encoding

```rust
use vb::{e_li, d_li};

fn main() {
    let list = vec![0, 1, 127, 128, 300, 16383, 16384];

    // Encode list
    let encoded = e_li(&list);
    println!("Encoded length: {} bytes", encoded.len());

    // Decode list
    let decoded = d_li(&encoded).unwrap();
    assert_eq!(list, decoded);
}
```

### Differential Encoding

Useful for sorted sequences like timestamps or primary keys.

```rust
#[cfg(feature = "diff")]
use vb::{e_diff, d_diff};

#[cfg(feature = "diff")]
fn main() {
    // Strictly increasing sequence
    let list = vec![10000, 10001, 10002, 10003, 10004];

    // Encode using differential encoding
    let encoded = e_diff(&list);

    // Decode back to original sequence
    let decoded = d_diff(&encoded).unwrap();
    assert_eq!(list, decoded);
}
```

## API Reference

The library exports the following key functions:

- **`d(input: impl AsRef<[u8]>) -> Result<(u64, usize)>`**
  Decodes a single variable-byte encoded integer. Returns the value and bytes consumed.

- **`e(value: u64, bytes: &mut Vec<u8>)`**
  Encodes a single `u64` into variable-byte format and appends to the buffer.


- **`e_li(li: impl AsRef<[u64]>) -> Vec<u8>`**
  Encodes a list of `u64` integers into a byte vector.

- **`d_li(data: impl AsRef<[u8]>) -> Result<Vec<u64>>`**
  Decodes a byte vector back into a list of `u64` integers.

- **`e_diff(li: impl AsRef<[u64]>) -> Vec<u8>`** *(feature: `diff`)*
  Encodes a strictly increasing sequence using differential encoding combined with VByte.

- **`d_diff(vs: impl AsRef<[u8]>) -> Result<Vec<u64>>`** *(feature: `diff`)*
  Decodes a differentially encoded sequence.

## Design

The VByte format uses 7 bits of each byte for data and the Most Significant Bit (MSB) as a continuation flag.
- **MSB = 0**: The last byte of the integer.
- **MSB = 1**: More bytes follow.

For differential encoding (`e_diff`), the library first calculates the difference between adjacent elements ($x_i - x_{i-1}$) and then encodes these smaller differences using VByte. This results in significant compression for dense, increasing sequences.

## Tech Stack
- **Rust**: Core language.
- **thiserror**: For ergonomic error handling.

## Directory Structure

```
.
├── Cargo.toml      # Project configuration
├── readme          # Documentation
│   ├── en.md       # English README
│   └── zh.md       # Chinese README
├── src
│   └── lib.rs      # Source code
└── tests
    └── main.rs     # Integration tests
```

## History

Variable Byte encoding (also known as **VByte**, **Varint**, or **LEB128**) has a rich history in computer science.

- **MIDI Standard**: One of the earliest widespread uses was in the MIDI file format as "Variable-Length Quantity" (VLQ) to save space in music files.
- **Search Engines**: In the late 90s, search engines like Google used VByte to compress inverted indexes (lists of document IDs), balancing high compression ratios with very fast decoding speeds.
- **DWARF**: The DWARF debugging data format uses a variant called LEB128 (Little Endian Base 128).
- **Protocol Buffers**: Google's Protocol Buffers heavily rely on "Varints" for efficient data serialization.

This simple yet powerful technique remains a cornerstone of data compression in systems where both space and speed matter.