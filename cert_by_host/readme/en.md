# cert_by_host : Dynamic SSL Certificate Loading with Auto-Expiration

`cert_by_host` is a high-performance Rust library designed to dynamically load HTTPS certificates from Kvrocks (a Redis-compatible store) based on hostnames. It features an efficient in-memory cache with automatic expiration handling, ensuring your application serves the correct certificates with minimal latency.

## Table of Contents

- [Features](#features)
- [Usage](#usage)
- [Design Architecture](#design-architecture)
- [Tech Stack](#tech-stack)
- [Project Structure](#project-structure)
- [API Reference](#api-reference)
- [The Story](#the-story)

## Features

- **Dynamic Loading**: Fetches SSL certificates on-demand from Kvrocks using the hostname as the key.
- **High-Performance Caching**: Utilizes `papaya` for concurrent, lock-free reads, ensuring extremely fast access during TLS handshakes.
- **Automatic Expiration**: A background task periodically cleans up expired certificates, preventing memory leaks and ensuring validity.
- **Cache Penetration Protection**: Remembers non-existent hosts for a short period to avoid repeated database hits for invalid domains.
- **Rustls Integration**: Directly returns `rustls` compatible types (`PrivateKeyDer`, `CertificateDer`), ready for immediate use.

## Usage

Add `cert_by_host` to your `Cargo.toml`. Ensure you have a Kvrocks/Redis instance running and configured.

```rust
use cert_by_host::get;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the system (spawns background tasks)
    xboot::init().await?;

    let host = "example.com";

    // Retrieve the certificate configuration
    if let Some(ssl_config_ref) = get(host).await? {
        println!("Loaded certificate for: {}", host);
        // Use ssl_config_ref.key and ssl_config_ref.cert with your TLS acceptor
    } else {
        println!("No certificate found for: {}", host);
    }

    Ok(())
}
```

## Design Architecture

The library employs a multi-layered strategy to balance performance and freshness:

1.  **L1 Cache (Memory)**: When `get(host)` is called, it first checks a global `papaya::HashMap`. If present, it returns a clone immediately.
2.  **Negative Cache**: If the host is known to not exist (checked via `ExpireSet`), it returns `None` immediately to protect the backend.
3.  **L2 Storage (Kvrocks)**: If not in memory, it queries Kvrocks.
    - **Data Format**: Expects a JSON string containing the private key and certificate chain in PEM format.
    - **Parsing**: Parses the PEM data into `rustls` types and extracts the expiration date using `x509_parser`.
4.  **Expiration Management**:
    - Upon successful load, the expiration timestamp is added to a `BTreeMap` (sorted by time).
    - A background loop runs every hour. It efficiently queries the `BTreeMap` for deadlines that have passed and removes the corresponding entries from the main `papaya::HashMap`.

## Tech Stack

- **Core**: Rust
- **Async Runtime**: `tokio`
- **Caching**: `papaya` (concurrent hash map), `expire_set`
- **TLS**: `rustls`, `rustls-pemfile`
- **Database**: `fred` (Redis client), `xkv` wrapper
- **Serialization**: `sonic_rs` (fast JSON)
- **Time**: `coarsetime`
- **Certificate Parsing**: `x509_parser`

## Project Structure

```
.
├── src
│   ├── lib.rs            # Core logic: caching, expiration task, public API
│   ├── get_by_kvrocks.rs # Database interaction and certificate parsing
│   └── error.rs          # Custom error definitions
├── tests
│   └── main.rs           # Integration tests
└── Cargo.toml
```

## API Reference

### `pub async fn get(host: impl Into<String>) -> Result<Option<Cert>>`

The main entry point.

- **Input**: A hostname string (e.g., "example.com").
- **Output**: A `Result` containing an `Option`.
  - `Some(Cert)`: A wrapper holding `Arc<SslConfig>`, implements `Deref` for direct access.
  - `None`: No certificate found for this host.

### `pub struct SslConfig`

Holds the parsed cryptographic material.

- `pub key: PrivateKeyDer<'static>`: The private key.
- `pub cert: Vec<CertificateDer<'static>>`: The certificate chain.

## The Story

In the era of SaaS and PaaS, platforms often manage tens of thousands of custom domains for their users. Loading all certificates into memory at startup is inefficient and slow. `cert_by_host` was born out of the need to serve SSL certificates dynamically and instantly. By combining the speed of in-memory caching with the persistence of Kvrocks, it solves the "C10K" problem for SSL termination, ensuring that even with millions of domains, your server only holds what's currently active in memory.
