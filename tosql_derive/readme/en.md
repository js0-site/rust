# tosql_derive : Derive Macro for tosql

Automatically implements the `ToSqlTrait` trait for Rust structs, simplifying SQL serialization.

## Table of Contents

- [Introduction](#introduction)
- [Features](#features)
- [Usage](#usage)
- [Design](#design)
- [Tech Stack](#tech-stack)
- [Directory Structure](#directory-structure)

## Introduction

`tosql_derive` is a procedural macro that generates the boilerplate code required to implement the `tosql::ToSqlTrait` trait. It analyzes your struct fields and generates the appropriate serialization logic, mapping Rust types to `kind2sql::Kind` variants.

## Features

- **Automatic Derivation**: Simply add `#[derive(ToSql)]` to your struct.
- **Type Mapping**: Automatically maps Rust primitive types (`u8`, `i32`, `String`, etc.) to SQL types.
- **Variable Byte Encoding**: Handles efficient encoding for `String`, `Bytes`, `u32`, and `u64`.
- **Seamless Integration**: Designed to work perfectly with the `tosql` and `kind2sql` ecosystem.

## Usage

Add `tosql` and `tosql_derive` to your dependencies:

```toml
[dependencies]
tosql = "0.1"
tosql_derive = "0.1"
```

Then derive `ToSql` on your struct:

```rust
use tosql_derive::ToSql;

#[derive(ToSql)]
struct User {
    id: u64,
    username: String,
    is_active: u8,
}

// Now User implements ToSqlTrait automatically
// You can use it with kind2sql to generate SQL inserts
```

## Design

The macro parses the struct definition and performs the following mappings:

- **Integers**: Mapped to their corresponding `Kind` (e.g., `u8` -> `Kind::U8`). `u32` and `u64` are treated as variable-byte encoded integers to save space.
- **Strings**: Mapped to `Kind::String`. The length is encoded using variable-byte encoding before the string data.
- **Bytes**: `Vec<u8>`, `Bytes`, and `bytes::Bytes` are mapped to `Kind::Bytes`.

The generated `dump` method efficiently writes these values into a `BytesMut` buffer without unnecessary copying.

## Tech Stack

- [Rust](https://www.rust-lang.org/)
- `syn`: For parsing Rust code.
- `quote`: For generating Rust code.
- `proc-macro2`: For manipulating token streams.

## Related Crates

- [tosql](https://docs.rs/crate/tosql): The core trait definition.
- [to_mysql](https://docs.rs/crate/to_mysql): MySQL SQL generation logic.
- [tosql_meta](https://docs.rs/crate/tosql_meta): Metadata definition for SQL structs.

## Directory Structure

```
.
├── Cargo.toml      # Package manifest
├── readme/         # Documentation
│   ├── en.md       # English README
│   └── zh.md       # Chinese README
└── src/
    └── lib.rs      # Macro implementation
```
