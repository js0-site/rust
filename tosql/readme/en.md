# tosql : Trait for SQL Struct Serialization

Defines the `ToSqlTrait` trait for serializing Rust structs into binary formats compatible with SQL bulk inserts.

## Table of Contents

- [Introduction](#introduction)
- [Features](#features)
- [Usage](#usage)
- [API Reference](#api-reference)
- [Tech Stack](#tech-stack)
- [Directory Structure](#directory-structure)

## Introduction

`tosql` provides the fundamental `ToSqlTrait` trait used to map Rust structs to SQL table schemas. It works in conjunction with `kind2sql` to enable efficient, type-safe serialization of data for database operations.

## Features

- **Standardized Trait**: Defines a common interface for SQL-compatible structs.
- **Type Safety**: Leverages `kind2sql::Kind` to ensure data types match the database schema.
- **Zero-Copy Friendly**: Designed to work with `bytes::Bytes` for efficient memory usage.

## Usage

The most common way to use `tosql` is with the `tosql_derive` macro to automatically implement the `ToSqlTrait` trait.

First, add dependencies:

```toml
[dependencies]
tosql = "0.1"
tosql_derive = "0.1"
```

Then, derive `ToSql` for your structs:

```rust
use tosql::{SqlField, ToSqlTrait, ToSql, mysql::{KIND, Mysql}};

#[derive(ToSql, Debug)]
struct User {
  id: u64,
  name: String,
  age: u8,
}

fn main() {
  let user = User {
    id: 1001,
    name: "Alice".to_string(),
    age: 30,
  };

  // 1. Get Schema Information
  println!("Table Name: {}", User::name());
  println!("Fields: {:?}", User::field_li());
  println!("Types: {:?}", User::kind_li());

  // 2. Serialize Data
  let bytes = user.dump();
  println!("Serialized bytes: {:?}", bytes);

  // 3. Convert to SQL Values (using kind2sql's Mysql implementation)
  let sql_values = Mysql::sql_field(&User::kind_li(), bytes).unwrap();
  println!("SQL Values: {:?}", sql_values); 
  // Output: ["1001", "'Alice'", "30"]

  // 4. Generate SQL Statement Example
  let columns = User::field_li().join(", ");
  let values = sql_values.join(", ");
  println!("INSERT INTO `{}` ({}) VALUES ({});", User::name(), columns, values);
}
```

## API Reference

### `trait ToSqlTrait`

- `fn name() -> String`: Returns the struct (or table) name.
- `fn kind_li() -> Vec<Kind>`: Returns the list of field types (`Kind`).
- `fn field_li() -> Vec<String>`: Returns the list of field names.
- `fn dump(&self) -> Bytes`: Serializes the struct instance into a binary buffer.

## Tech Stack

- [Rust](https://www.rust-lang.org/)
- `bytes`: For efficient byte buffer management.
- `kind2sql`: For type definitions (`Kind`).

## Related Crates

- [tosql_derive](https://docs.rs/crate/tosql_derive): Macro to derive `ToSqlTrait`.
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
    └── lib.rs      # Trait definition
```