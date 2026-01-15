# kind2sql : Type-safe Binary to SQL Converter

A Rust library for efficiently converting a stream of serialized binary data into a sequence of SQL-compatible string values.

## Table of Contents

- [Introduction](#introduction)
- [Features](#features)
- [Usage](#usage)
- [Design](#design)
- [API Reference](#api-reference)
- [Tech Stack](#tech-stack)
- [Directory Structure](#directory-structure)
- [A Little Story](#a-little-story)

## Introduction

`kind2sql` provides a mechanism to map structured binary data to a format ready for SQL `INSERT` statements. It is designed for scenarios where performance is critical, such as bulk-importing logs or other serialized records from a byte stream directly into a database without intermediate parsing into high-level structs.

## Features

- **High Performance**: Operates directly on byte buffers (`Buf`), minimizing allocations and copying.
- **Type-Safe Schema**: Uses an enum (`Kind`) to define the "schema" of the binary data, preventing deserialization errors.
- **Extensible**: Supports different SQL dialects through the `SqlField` trait. A MySQL implementation is provided out-of-the-box.
- **Efficient Encoding**: Uses variable-byte encoding for string and byte array lengths, saving space.
- **Feature Gated**: Core functionality is split into features (`sql_field`, `mysql`) to keep the dependency tree minimal.

## Usage

First, add `kind2sql` to your `Cargo.toml` and enable the desired features. For example, to use the MySQL converter:

```toml
[dependencies]
kind2sql = { version = "0.1.0", features = ["mysql"] }
```

Next, define your data schema with `Kind` and pass your serialized data to the `sql_field` function.

```rust
use kind2sql::{Kind, SqlField, mysql::Mysql};

// 1. Define the data schema.
let kinds = [Kind::U8, Kind::I16, Kind::String];

// 2. Prepare the serialized data buffer.
let mut data = vec![];
// U8: 123
data.push(123u8);
// I16: -456 (little endian)
data.extend_from_slice(&(-456i16).to_le_bytes());
// String: "hello" (length-prefixed)
data.push(5u8); // vbyte encoded length
data.extend_from_slice(b"hello");

// 3. Convert the data.
let result = Mysql::sql_field(&kinds, &data[..]).unwrap();

assert_eq!(result, vec!["123", "-456", "'hello'"]);
```

The output `Vec<String>` contains values that are properly formatted and escaped for a MySQL `INSERT` statement.

## Design

The library's design revolves around three main components:

1.  **`Kind` Enum**: This enum acts as a schema descriptor. An array of `Kind` variants defines the sequence and type of data fields packed in the binary buffer.

2.  **`sql_field` function**: This internal function is the engine of the library. It reads from a type implementing `bytes::Buf` and, based on the `Kind` provided, deserializes one value, converts it to a string, and advances the buffer. For `String` and `Bytes`, it first decodes a variable-byte integer to determine the length of the upcoming data.

3.  **`SqlField` Trait**: This trait abstracts the dialect-specific formatting, particularly for binary data (`BLOB`). To support a new database (e.g., PostgreSQL), you would implement `SqlField` and provide a `blob` function that formats byte arrays according to that database's requirements.

The overall process is:
`SqlField::sql_field` -> loops through `&[Kind]` -> calls internal `sql_field` for each `Kind` -> `sql_field` reads from `Buf`, formats data, and pushes to output vector.

## API Reference

### `enum Kind`

Defines the supported data types that can be deserialized from the byte buffer.

- `U8`, `I8`: 1-byte integers
- `U16`, `I16`: 2-byte little-endian integers
- `U32`: Variable-byte encoded unsigned 32-bit integer (1-5 bytes)
- `I32`: 4-byte little-endian signed integer
- `U64`: Variable-byte encoded unsigned 64-bit integer (1-10 bytes)
- `I64`: 8-byte little-endian signed integer
- `String`: A UTF-8 string, prefixed with its `vbyte`-encoded length
- `Bytes`: A byte array, prefixed with its `vbyte`-encoded length

### `trait SqlField`

A trait to be implemented by dialect-specific converters.

- `fn blob(data: &[u8]) -> String`: A required function that defines how a byte slice should be formatted for the target SQL dialect (e.g., `X'AABBCC'` for MySQL).
- `fn sql_field(...)`: The primary method that orchestrates the conversion process using the provided kinds and buffer.

### `mysql::Mysql`

A concrete implementation of `SqlField` for MySQL, available under the `mysql` feature flag.

## Tech Stack

- [Rust](https://www.rust-lang.org/)
- `bytes`: For efficient buffer manipulation.
- `num_enum`: For converting the `Kind` enum from/to integers.
- `vb`: For `vbyte` encoding/decoding of lengths.
- `sqle`: For SQL string and blob escaping.

## Directory Structure

```
.
├── Cargo.toml      # Package manifest
├── AGENTS.md       # Agent instructions
├── readme/         # Documentation
│   ├── en.md       # English README
│   └── zh.md       # Chinese README
├── src/
│   ├── lib.rs      # Main library file, exports modules and types
│   └── sql_field.rs   # Core conversion logic and `SqlField` trait
└── tests/
    └── main.rs     # Integration tests
```

## A Little Story

The term "BLOB" (Binary Large Object) was coined by Jim Starkey at Digital Equipment Corporation (DEC) in the 1980s. According to Starkey, he had just watched the 1958 horror film *The Blob*, which features a gelatinous, amorphous alien that consumes everything in its path. He thought it was a fitting name for the data type he was working on, which was designed to store large, unstructured chunks of binary data in a database. The name stuck and is now a standard part of SQL, reminding us that even in the structured world of databases, there's a place for a bit of amorphous creativity.
