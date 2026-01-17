# clap_args : Lightweight Clap Wrapper for Automatic Version & Help Handling

A simplified wrapper around `clap` that streamlines command-line argument parsing by automatically handling version and help flags.

## Table of Contents

- [Features](#features)
- [Usage](#usage)
- [Design Philosophy](#design-philosophy)
- [Tech Stack](#tech-stack)
- [Directory Structure](#directory-structure)
- [API Reference](#api-reference)
- [History](#history)

## Features

- **Automatic Versioning**: Reads version directly from `Cargo.toml`.
- **Detailed Info**: Provides platform and target information via `--vv`.
- **Built-in Help**: Automatically handles `-h` and `--help` flags.
- **Macro Support**: The `parse!` macro simplifies initialization.

## Usage

Add to `Cargo.toml`:

```toml
[dependencies]
clap_args = { version = "*", features = ["macro"] }
```

### Example

```rust
use clap_args::{arg, ArgAction};

fn main() -> aok::Result<()> {
  if let Some(matches) = clap_args::parse!(|cmd| {
    cmd
      .arg(arg!(-b --bind [BIND] "http proxy bind address").default_value("0.0.0.0:15080"))
      .arg(arg!(-p --port <PORT> "listen port"))
      .arg(arg!(-d --debug "enable debug mode").action(ArgAction::SetTrue))
  }) {
    let bind: &String = matches.get_one("bind").unwrap();
    println!("bind: {bind}");

    if let Some(port) = matches.get_one::<String>("port") {
      println!("port: {port}");
    }

    if matches.get_flag("debug") {
      println!("debug mode enabled");
    }
  }
  Ok(())
}
```

## Design Philosophy

The core design goal is to reduce boilerplate for common CLI tasks.

**Call Flow**:
1.  **Macro Invocation**: `parse!` is called with a closure.
2.  **Setup**: The `parse` function initializes a `clap::Command` with default flags (`-v`, `--vv`, `-h`).
3.  **User Configuration**: The user-provided closure configures additional arguments.
4.  **Execution**: `clap` parses the arguments.
5.  **Handling**:
    *   If version or help flags are present, they are handled immediately, and `None` is returned.
    *   Otherwise, `Some(ArgMatches)` is returned for the user to process.

## Tech Stack

- **Rust**: Core language.
- **Clap**: Underlying argument parser.
- **const_str**: Compile-time string manipulation.
- **current_platform**: Platform information retrieval.

## Directory Structure

```
.
├── Cargo.toml      # Project configuration
├── examples/       # Usage examples
├── readme/         # Documentation
├── src/            # Source code
│   └── lib.rs      # Main library file
└── test.sh         # Test script
```

## API Reference

### `parse!` Macro

Initializes the parser with the current package name and version.

```rust
clap_args::parse!(|cmd| { ... })
```

### `parse` Function

The underlying function called by the macro.

```rust
pub fn parse(
  project: impl Into<String>,
  ver: impl Borrow<[u64; 3]>,
  cmd_build: impl FnOnce(Command) -> Command,
) -> Option<ArgMatches>
```

### Exports

- `arg!`: Re-exported from `clap` for defining arguments.
- `ArgAction`: Re-exported from `clap` for defining argument actions.

## History

The concept of command-line arguments dates back to the early days of Unix in the 1970s. The `argv` (argument vector) convention allowed programs to receive input dynamically at runtime, a significant leap from hardcoded parameters. Over decades, parsing libraries evolved from simple loop-based checks to sophisticated frameworks like `clap` in Rust, which offer type safety, auto-generated help, and subcommands, reflecting the growing complexity and capability of modern CLI tools.