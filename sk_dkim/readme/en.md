# sk_dkim : Deterministic DKIM Key Generation

## Table of Contents

- [Introduction](#introduction)
- [Usage](#usage)
- [Design](#design)
- [API Reference](#api-reference)
- [Tech Stack](#tech-stack)
- [Directory Structure](#directory-structure)
- [History & Trivia](#history--trivia)

## Introduction

`sk_dkim` is a Rust library designed to generate **DomainKeys Identified Mail (DKIM)** keys and DNS TXT records deterministically. Instead of managing and storing private key files for every domain, you can derive the necessary Ed25519 keys on-the-fly using a single secret seed (Secret Key) combined with the domain name and selector.

This approach simplifies key management, especially for services managing DKIM for multiple domains, as it eliminates the need for stateful storage of private keys.

## Usage

Add `sk_dkim` to your `Cargo.toml`:

```toml
[dependencies]
sk_dkim = { version = "0.1.1", features = ["pk"] }
```

> Note: The `pk` feature is required to generate the formatted TXT record string.

### Example

```rust
use sk_dkim::Sk;

fn main() {
    // Your secret seed (keep this safe!)
    let secret_seed = "your_secret_seed_string";
    
    // Initialize the generator with the seed
    let sk = Sk::new(secret_seed);

    let selector = "default";
    let domain = "example.com";

    // Generate the DKIM struct for the specific domain and selector
    let dkim = sk.dkim(selector, domain);

    // Get the DNS TXT record value
    // Output format: v=DKIM1;k=ed25519;p=...
    println!("DKIM Record: {}", dkim.txt());
}
```

## Design

The core philosophy of `sk_dkim` is **determinism**.

1.  **Initialization**: The `Sk` struct is initialized with a base secret seed. This seed initializes a `BLAKE3` hasher.
2.  **Derivation**: When `dkim(selector, domain)` is called, the hasher is cloned and updated with the `selector` and `domain`.
3.  **Key Generation**: The final hash digest is used as the seed to generate an **Ed25519** signing key.
4.  **Output**: The public part of the key is encoded in Base64 and formatted into a standard DKIM TXT record.

This process ensures that as long as the secret seed remains constant, the generated DKIM keys for any given domain will always be the same.

## API Reference

### `struct Sk`

The main entry point for key generation.

*   **`Sk::new(sk: impl AsRef<[u8]>) -> Self`**
    Creates a new `Sk` instance using the provided secret seed.

*   **`Sk::dkim(&self, selector: impl AsRef<str>, domain: impl AsRef<str>) -> Dkim`**
    Derives a `Dkim` instance for the specified selector (`selector`) and domain (`domain`).

### `struct Dkim`

Represents the generated DKIM key pair.

*   **`pub sk: ed25519_dalek::SigningKey`**
    The underlying Ed25519 signing key.

*   **`Dkim::txt(&self) -> String`**
    *(Requires `pk` feature)*
    Returns the formatted DKIM DNS TXT record string (e.g., `v=DKIM1;k=ed25519;p=...`).

## Tech Stack

*   **Rust**: Core language.
*   **ed25519-dalek**: Fast and secure Ed25519 key generation and signing.
*   **blake3**: Cryptographic hashing for deterministic key derivation.
*   **base64**: Encoding the public key for DNS records.

## Directory Structure

```
.
├── Cargo.toml      # Project configuration and dependencies
├── readme/         # Documentation
│   ├── en.md       # English README
│   └── zh.md       # Chinese README
├── src/            # Source code
│   └── lib.rs      # Library entry point and implementation
├── tests/          # Integration tests
│   └── main.rs     # Usage demonstration and testing
└── test.sh         # Test execution script
```

## History & Trivia

**DKIM (DomainKeys Identified Mail)** was formed in 2007 through the merger of two separate email authentication protocols: **DomainKeys** by Yahoo! and **Identified Internet Mail (IIM)** by Cisco. It became an IETF standard in 2011 (RFC 6376).

**Ed25519**, the algorithm used by this library, is a high-performance public-key signature system introduced in 2011 by Daniel J. Bernstein and his team. It is based on **Curve25519** and offers significant security and performance advantages over older algorithms like RSA.

The intersection of these two technologies occurred with **RFC 8463** in 2018, which officially added support for Ed25519 in DKIM signatures. This was a significant step forward, as Ed25519 keys are much shorter than RSA keys of equivalent strength, making them far easier to fit into DNS TXT records without hitting size limits.