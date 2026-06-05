# password\_ : Secure Password Hashing Made Simple

A lightweight and secure password hashing library based on Argon2.

## Table of Contents

- [Introduction](#introduction)
- [Usage](#usage)
- [Design](#design)
- [Tech Stack](#tech-stack)
- [Directory Structure](#directory-structure)
- [API Reference](#api-reference)
- [History](#history)

## Introduction

`password_` provides a simplified interface for hashing and verifying passwords using the state-of-the-art Argon2 algorithm. It abstracts away complex configuration, offering secure defaults for immediate use.

## Usage

See `tests/main.rs` for a complete demonstration.

```rust
use aok::{OK, Void};
use log::info;

fn main() -> Void {
  let password = "test";
  // Generate a random salt and hash the password
  let (salt, hash) = password_::hash(password);

  info!("{salt:?} {hash:?}");

  // Verify the password against the salt and hash
  assert!(password_::verify(password, &salt, &hash));

  OK
}
```

## Design

The library uses a static configuration for Argon2 to ensure consistency and security:

- **Algorithm**: Argon2id (hybrid version, resistant to GPU and side-channel attacks).
- **Version**: 0x13.
- **Memory**: 64 MB (65536 KB).
- **Iterations**: 3.
- **Parallelism**: 1.
- **Output Length**: 32 bytes.

The `hash` function generates a random 16-byte salt and computes the 32-byte hash. The `verify` function re-computes the hash using the provided salt and compares it with the stored hash.

## Tech Stack

- **Language**: Rust
- **Core Algorithm**: `argon2` crate
- **Randomness**: `rand` crate
- **Initialization**: `static_init` crate

## Directory Structure

```
.
├── src/
│   └── lib.rs       # Core logic and API definitions
├── tests/
│   └── main.rs      # Usage demonstration and tests
├── readme/          # Documentation
└── Cargo.toml       # Project configuration
```

## API Reference

The library exports the following from `lib.rs`:

### Types

- `SALT`: Alias for `[u8; 16]`.
- `HASH`: Alias for `[u8; 32]`.

### Functions

- `fn hash(password: impl AsRef<[u8]>) -> (SALT, HASH)`
  Generates a random salt and returns the (salt, hash) tuple.

- `fn hash_with_salt(password: impl AsRef<[u8]>, salt: impl AsRef<[u8]>) -> HASH`
  Computes the hash for a given password and salt.

- `fn verify(password: impl AsRef<[u8]>, salt: impl AsRef<[u8]>, hash: impl AsRef<[u8]>) -> bool`
  Verifies if the password matches the provided salt and hash.

## History

In 2013, the Password Hashing Competition (PHC) was launched to find a successor to aging algorithms like PBKDF2 and bcrypt, which were becoming vulnerable to GPU-based attacks. After two years of rigorous analysis, **Argon2** was selected as the winner in July 2015. Designed by Alex Biryukov, Daniel Dinu, and Dmitry Khovratovich, Argon2 introduced memory-hardness properties that make it prohibitively expensive to crack using specialized hardware, setting a new standard for password security.
