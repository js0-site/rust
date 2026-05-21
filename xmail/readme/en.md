# xmail : Normalize and Parse Email Addresses with Ease

[English](readme/en.md) | [中文](readme/zh.md)

## Table of Contents

- [Introduction](#introduction)
- [Features](#features)
- [Usage](#usage)
- [Design](#design)
- [Tech Stack](#tech-stack)
- [Directory Structure](#directory-structure)
- [API Reference](#api-reference)
- [History](#history)

## Introduction

`xmail` is a lightweight Rust library designed to parse and normalize email addresses. It handles common tasks such as splitting the user and host parts, converting to lowercase, trimming whitespace, and decoding Punycode for internationalized domain names (IDN). It also offers optional support for extracting the Top-Level Domain (TLD).

## Features

- **Split User and Host**: Efficiently separates the local part (user) from the domain part (host).
- **Normalization**: Automatically trims whitespace and converts the email address to lowercase.
- **Punycode Support**: Decodes Punycode-encoded domains (e.g., `xn--...`) into their Unicode representation.
- **TLD Extraction**: Optionally extracts the effective Top-Level Domain (requires `tld` feature).
- **Robust Parsing**: Handles edge cases like missing `@` symbols, empty user parts, or invalid domains (missing `.`, or starting/ending with `.`) gracefully using `Option`.

## Usage

Add `xmail` to your `Cargo.toml`:

```toml
[dependencies]
xmail = "0.1.15"
# For TLD support:
# xmail = { version = "0.1.15", features = ["tld"] }
```

### Examples

```rust
fn main() {
    // Basic normalization
    let email = "  User@Example.COM  ";
    let normalized = xmail::norm(email).unwrap();
    assert_eq!(normalized, "user@example.com");

    // Punycode decoding
    // xn--yfro4i67o is Punycode for "新加坡" (Singapore)
    let idn_email = "user@site.xn--yfro4i67o";
    let (user, host) = xmail::norm_user_host(idn_email).unwrap();
    assert_eq!(host, "site.新加坡");

    // TLD extraction (requires "tld" feature)
    #[cfg(feature = "tld")]
    {
        let (_, tld) = xmail::norm_tld("user@example.co.uk").unwrap();
        assert_eq!(tld, "co.uk");
    }
}
```

## Design

The library processes email addresses through a pipeline:

1.  **Input**: Accepts any string type that implements `AsRef<str>`.
2.  **Split**: The `user_host` function locates the first `@` symbol to separate the user and host parts.
3.  **Normalize**:
    - Trims leading/trailing whitespace.
    - Converts characters to lowercase.
    - Iterates through domain parts (separated by `.`) and decodes any starting with `xn--` using Punycode.
4.  **Output**: Returns the processed parts or the reconstructed email string wrapped in `Option`.

## Tech Stack

- **Rust**: The core language for safety and performance.
- **xstr**: Used for efficient string manipulation (trimming, cutting).
- **punycode**: Handles the decoding of Internationalized Domain Names.
- **xtld**: (Optional) Provides accurate Public Suffix List based TLD extraction.

## Directory Structure

```
.
├── Cargo.toml          # Project configuration
├── README.mdt          # Template for generating READMEs
├── readme/             # Documentation storage
│   ├── en.md           # English documentation
│   └── zh.md           # Chinese documentation
├── src/
│   └── lib.rs          # Core library logic
└── tests/
    └── main.rs         # Integration tests
```

## API Reference

### `user_host(mail: impl AsRef<str>) -> Option<(String, String)>`

Splits the email into user and host parts without normalization. Returns `None` if:

- `@` is missing.
- The user part is empty.
- The host part does not contain a dot (`.`).
- The host part starts or ends with a dot.

### `norm_user_host(mail: impl AsRef<str>) -> Option<(String, String)>`

Normalizes the email (lowercase, trim) and decodes Punycode in the host part.

### `norm(mail: impl AsRef<str>) -> Option<String>`

Normalizes the email and returns it as a single string `user@host`.

### `norm_tld(mail: impl AsRef<str>) -> Option<(String, String)>`

_Requires `tld` feature._ Normalizes the email and returns a tuple containing the full email address and the extracted TLD.

## History

The `@` symbol in email addresses was introduced by Ray Tomlinson in 1971 to separate the user from the host machine. He chose it simply because it was a rarely used character on the keyboard that made sense ("user" at "host").

The term **Punycode**, used in this library for international domains, has an interesting origin. It rhymes with "Unicode" and is "puny" in three ways: it uses a small character set (ASCII), generates short strings, and has a small implementation footprint. This clever encoding allows non-ASCII characters (like Chinese or Emoji) to exist within the legacy ASCII-only DNS system.
