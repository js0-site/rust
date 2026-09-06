# ipbin : Seamless IP Address to Binary Conversion

[中文](./zh.md)

A lightweight, zero-dependency Rust utility for converting IP addresses (IPv4 and IPv6) into their binary representation (byte vectors).

## Table of Contents

- [Features](#features)
- [Usage](#usage)
- [Design Philosophy](#design-philosophy)
- [Tech Stack](#tech-stack)
- [Directory Structure](#directory-structure)
- [API Documentation](#api-documentation)
- [Historical Anecdote](#historical-anecdote)

## Features

- **Universal Support**: Handles both IPv4 and IPv6 addresses seamlessly.
- **Zero Dependencies**: Built entirely on the Rust standard library (`std`).
- **Efficient**: Direct conversion using native Rust methods.
- **Simple API**: A single function to handle all conversions.

## Usage

Add `ipbin` to your project dependencies. Here is a quick example of how to use it:

```rust
use ipbin::ipbin;
use std::net::IpAddr;

fn main() {
    // IPv4 Example
    let ipv4: IpAddr = "127.0.0.1".parse().unwrap();
    let bytes_v4 = ipbin(ipv4);
    println!("IPv4 Bytes: {:?}", bytes_v4); // [127, 0, 0, 1]

    // IPv6 Example
    let ipv6: IpAddr = "::1".parse().unwrap();
    let bytes_v6 = ipbin(ipv6);
    println!("IPv6 Bytes: {:?}", bytes_v6); // [0, 0, ..., 1]
}
```

## Design Philosophy

The core design revolves around simplicity and leveraging Rust's strong type system. The `ipbin` function takes a `std::net::IpAddr` enum, which can be either `V4` or `V6`.

1.  **Input**: Accepts `IpAddr`.
2.  **Matching**: Matches the enum variant.
3.  **Conversion**: Calls `.octets()` on the inner IP struct.
4.  **Output**: Converts the array of octets into a `Vec<u8>`.

This ensures that the output is always a raw byte vector representing the IP address, regardless of its version.

## Tech Stack

- **Language**: Rust
- **Standard Library**: `std::net`

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

## API Documentation

### `ipbin`

```rust
pub fn ipbin(ipaddr: IpAddr) -> Vec<u8>
```

- **Parameters**: `ipaddr` - An `IpAddr` enum representing either an IPv4 or IPv6 address.
- **Returns**: `Vec<u8>` - A vector containing the octets of the IP address.
  - For IPv4, the vector length is 4.
  - For IPv6, the vector length is 16.

## Historical Anecdote

**The 32-bit "Experiment"**

In the 1970s, when Vint Cerf and his colleagues were designing the Internet Protocol (IPv4), they had to decide on the address size. After much debate, Cerf decided on 32 bits, which allows for about 4.3 billion unique addresses. At the time, this seemed like an astronomical number for what was essentially a military experiment. Cerf famously remarked, "It's enough for an experiment." He expected that if the experiment worked, a production version with a larger address space would be developed later.

That "experiment" escaped the lab and became the global Internet we know today. The 32-bit limit eventually led to address exhaustion, necessitating the creation of IPv6 with its massive 128-bit address space—which `ipbin` fully supports!
