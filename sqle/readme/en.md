# sqle : Concise SQL String Escaping and Formatting

`sqle` is a lightweight Rust library designed for safe and efficient SQL string escaping and binary data formatting. It provides simple utilities to prevent SQL injection and handle database-specific binary formats (MySQL and PostgreSQL).

## Table of Contents

- [Features](#features)
- [Usage](#usage)
- [Design](#design)
- [API Documentation](#api-documentation)
- [Tech Stack](#tech-stack)
- [Directory Structure](#directory-structure)
- [History](#history)

## Features

- **String Escaping**: Safely escapes special characters in SQL strings, including single quotes, backslashes, newlines, carriage returns, tabs, and null characters.
- **Boolean Formatting**: Converts Rust `bool` to SQL `TRUE` or `FALSE`.
- **MySQL Binary**: Formats byte arrays into MySQL `X'HEX'` literals.
- **PostgreSQL Binary**: Formats byte arrays into PostgreSQL `E'\\xHEX'` literals.
- **Extreme Performance**:
  - **Pre-allocation**: Calculates exact memory requirements (2-pass scan) to prevent expensive reallocations.
  - **Byte-level Processing**: Operates directly on `&[u8]` to avoid UTF-8 decoding overhead.
  - **Unsafe Optimization**: Uses `String::from_utf8_unchecked` for zero-overhead string construction.

## Usage

Add `sqle` to your `Cargo.toml`. Enable features as needed:

```toml
[dependencies]
sqle = { version = "0.1", features = ["mysql", "postgres"] }
```

### Examples

```rust
use sqle;

fn main() {
    // String Escaping - Single Quote
    let s = "foo'bar";
    assert_eq!(sqle::string(s), "'foo''bar'");

    // String Escaping - Newlines, Tabs, Backslashes, etc.
    let s = "hello\nworld\t!";
    assert_eq!(sqle::string(s), "'hello\\nworld\\t!'");

    // Boolean Formatting
    assert_eq!(sqle::bool(true), "TRUE");

    // MySQL Binary (requires "mysql" feature)
    #[cfg(feature = "mysql")]
    {
        let bytes = b"hello";
        assert_eq!(sqle::mysql::blob(bytes), "X'68656C6C6F'");
    }

    // PostgreSQL Binary (requires "postgres" feature)
    #[cfg(feature = "postgres")]
    {
        let bytes = b"hello";
        assert_eq!(sqle::postgres::blob(bytes), "E'\\\\x68656c6c6f'");
    }
}
```

## Design

The library focuses on simplicity and performance.

- **Minimal Allocation**: `string` function pre-allocates memory for worst-case scenario (all characters need escaping) to prevent runtime reallocations.
- **Feature Flags**: Database-specific implementations are gated behind `mysql` and `postgres` features to keep the core lightweight.
- **Safety**: Uses `unsafe` for string construction from known valid UTF-8 bytes (hex encoding) to squeeze out extra performance where safe.

## API Documentation

### `pub fn string(s: impl AsRef<[u8]>) -> String`

Escapes a string for use in a SQL query. It wraps the string in single quotes and escapes the following special characters:

- `'` → `''` (single quote doubled)
- `\` → `\\` (backslash)
- `\n` → `\n` (newline)
- `\r` → `\r` (carriage return)
- `\t` → `\t` (tab)
- `\0` → `\0` (null character)

This ensures safety and compatibility across various SQL dialects (MySQL, PostgreSQL, etc.).

### `pub fn bool(b: bool) -> &'static str`

Returns the SQL string representation of a boolean: `"TRUE"` or `"FALSE"`.

### `pub mod mysql`

Available with `feature = "mysql"`.

#### `pub fn blob(bytes: &[u8]) -> String`

Formats a byte slice into a MySQL hex string literal: `X'...'`.

### `pub mod postgres`

Available with `feature = "postgres"`.

#### `pub fn blob(bytes: &[u8]) -> String`

Formats a byte slice into a PostgreSQL hex string literal using the escape string syntax: `E'\\x...'`.

## Tech Stack

- **Rust**: Core language.
- **faster-hex**: High-performance hex encoding.

## Directory Structure

```
.
├── Cargo.toml      # Project configuration and dependencies
├── src
│   └── lib.rs      # Library source code
└── tests
    └── main.rs     # Integration tests
```

## History

**The Origin of SQL Injection**

In December 1998, a cybersecurity researcher known as "Rain Forest Puppy" (Jeff Forristal) published an article in _Phrack_ magazine (Issue 54). He detailed how he could "piggyback" SQL commands into legitimate queries through NT web servers running ODBC. This was the first formal documentation of **SQL Injection**.

Before this, the concept of "escaping" characters had existed since the 19th century (Baudot code), but the specific danger of mixing data and code in database queries wasn't widely recognized. Today, proper string escaping (like what `sqle` provides) and parameterized queries are the standard defense against this historic vulnerability.
