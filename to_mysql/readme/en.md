# to_mysql : Efficient MySQL SQL Generation

`to_mysql` is a Rust library designed to efficiently generate MySQL `CREATE TABLE` and `INSERT` statements from Rust structs. It leverages `tosql` and `kind2sql` to map Rust types to MySQL types and serialize data, providing a high-performance solution for SQL generation.

## Table of Contents

- [Introduction](#introduction)
- [Features](#features)
- [Usage](#usage)
- [Design](#design)
- [Tech Stack](#tech-stack)
- [Directory Structure](#directory-structure)
- [API Documentation](#api-documentation)
- [History](#history)

## Introduction

`to_mysql` simplifies the process of interacting with MySQL databases by automating the generation of SQL statements. It takes the schema definition and data from Rust structs and produces optimized SQL strings, ready for execution.

## Features

- **Automatic Schema Generation**: Creates `CREATE TABLE` statements based on struct fields and types.
- **Efficient Data Insertion**: Generates `INSERT` statements with precomputed prefixes for maximum performance.
- **Type Safety**: Ensures Rust types are correctly mapped to MySQL types.
- **Caching**: Caches the `INSERT` statement prefix to reduce overhead during repetitive insertions.

## Usage

Add `to_mysql` to your `Cargo.toml`.

```rust
use to_mysql::Mysql;
use tosql::{ToSqlTrait, ToSql};

#[derive(ToSql)]
struct User {
  id: u64,
  name: String,
}

fn main() {
  // Generate CREATE TABLE statement
  let mysql = Mysql::new("User", User::meta());
  let create_table_sql = mysql.create_table();
  println!("{}", create_table_sql);
  // Output: CREATE TABLE User(id BIGINT UNSIGNED,name LONGTEXT);

  // Generate INSERT statement
  let user = User {
    id: 123,
    name: "Alice".to_string(),
  };
  let insert_sql = mysql.insert(&user.dump()).unwrap();
  println!("{}", insert_sql);
  // Output: INSERT INTO User(id,name)VALUES(123,'Alice');
}
```

## Design

The core of `to_mysql` is the `Mysql` struct.

1.  **Initialization (`new`)**: When a `Mysql` instance is created, it takes the table name and schema information (kinds and field names). It precomputes the `INSERT INTO ... VALUES(` prefix string. This optimization avoids rebuilding the static part of the SQL query for every insertion.
2.  **Schema Mapping**: The `create_table` method iterates over the field names and kinds, mapping each Rust type (via `Kind`) to its corresponding MySQL type string (e.g., `u64` -> `BIGINT UNSIGNED`).
3.  **Data Serialization**: The `insert` method takes serialized byte data, converts it into SQL-compatible string values using `kind2sql`, and appends them to the precomputed prefix.

## Tech Stack

- **Rust**: The core programming language.
- **tosql**: Provides the `ToSql` trait and `ToSqlTrait` for struct serialization and metadata.
- **kind2sql**: Handles the mapping between Rust types (`Kind`) and SQL types, as well as value formatting.

## Related Crates

- [tosql](https://docs.rs/crate/tosql): The core trait definition.
- [tosql_derive](https://docs.rs/crate/tosql_derive): Macro to derive `ToSqlTrait`.
- [tosql_meta](https://docs.rs/crate/tosql_meta): Metadata definition for SQL structs.

## Directory Structure

```
to_mysql/
├── Cargo.toml      # Project configuration and dependencies
├── src/
│   └── lib.rs      # Core logic and Mysql struct implementation
└── tests/
    └── main.rs     # Integration tests demonstrating usage
```

## API Documentation

### `Mysql` Struct

The main entry point for the library.

#### `pub fn new(table_name: impl Into<String>, (kind_li, field_li): (Vec<Kind>, Vec<String>)) -> Self`

Creates a new `Mysql` instance.

- `table_name`: The name of the MySQL table.
- `kind_li`: A vector of `Kind` enums representing the types of the fields.
- `field_li`: A vector of strings representing the names of the fields.

#### `pub fn create_table(&self) -> String`

Generates a `CREATE TABLE` SQL statement.

- Returns: A string containing the SQL statement.

#### `pub fn insert(&self, bytes: &[u8]) -> tosql::vb::Result<String>`

Generates an `INSERT` SQL statement for a specific record.

- `bytes`: The serialized byte representation of the struct (from `ToSqlTrait::dump`).
- Returns: A `Result` containing the SQL string or an error.

## History

**The Name "MySQL"**

MySQL was created by a Swedish company, MySQL AB, founded by David Axmark, Allan Larsson, and Michael "Monty" Widenius. The "My" in MySQL is named after Monty's daughter, My. The dolphin logo, named "Sakila," was chosen from a huge list of names suggested by users in a "Name the Dolphin" contest. The project started in 1994, and the first version was released on May 23, 1995. It was designed to be a faster, more flexible alternative to existing database systems like mSQL.
