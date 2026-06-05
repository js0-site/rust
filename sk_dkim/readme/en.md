# sk_dkim : Deterministic DKIM Key Generation

## Table of Contents

- [Introduction](#introduction)
- [Why RSA?](#why-rsa-the-sad-state-of-ed25519-support)
- [Usage](#usage)
- [Design](#design)
- [API Reference](#api-reference)
- [Tech Stack](#tech-stack)
- [Directory Structure](#directory-structure)

## Introduction

`sk_dkim` is a Rust library designed to generate **DomainKeys Identified Mail (DKIM)** keys and DNS TXT records deterministically. Instead of managing and storing private key files for every domain, you can derive the necessary **RSA 2048** keys on-the-fly using a single secret seed (Secret Key) combined with the domain name and selector.

This approach simplifies key management, especially for services managing DKIM for multiple domains, as it eliminates the need for stateful storage of private keys.

## Why RSA? (The Sad State of Ed25519 Support)

Technically, **Ed25519** is superior to RSA in almost every way for DKIM:

- **Size**: Ed25519 public keys are tiny (32 bytes), fitting easily into a single DNS TXT record. RSA 2048 keys are huge and often require splitting across multiple strings in a TXT record.
- **Performance**: Ed25519 signing and verification are incredibly fast.
- **Security**: Ed25519 offers high security with smaller key sizes.

**RFC 8463**, published in **2018**, explicitly states that verifiers **MUST** implement Ed25519 verification. However, the reality of the email ecosystem in 2024/2025 is a disappointing display of non-compliance by the industry giants:

- **Gmail**: Support is inconsistent at best. While they claim some support, verification frequently fails for inbound emails signed with Ed25519.
- **Outlook / Microsoft 365**: They **consistently fail** to verify Ed25519 signatures. If you send an email to an Outlook/Hotmail address with only an Ed25519 signature, it will likely be treated as unsigned or fail authentication.
- **Yahoo**: Similar to Microsoft, Yahoo Mail typically fails to verify inbound Ed25519 DKIM signatures.

Despite the standard being over 6 years old, these major providers have effectively ignored the "MUST implement" clause for verification. While some experts recommend "dual signing" (signing with both RSA and Ed25519), this adds significant complexity and bloat to headers.

Because of this widespread lack of support, `sk_dkim` was forced to switch from Ed25519 to **RSA 2048**. We had to compromise on elegance and efficiency to ensure deliverability in the real world. It's a step backward in technology, but a necessary one for practical usage.

## Usage

Add `sk_dkim` to your `Cargo.toml`:

```toml
[dependencies]
sk_dkim = { version = "0.1.5", features = ["pk"] }
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
    // Output format: v=DKIM1; k=rsa; p=...
    println!("DKIM Record: {}", dkim.txt());
}
```

## Design

The core philosophy of `sk_dkim` is **determinism**.

1.  **Initialization**: The `Sk` struct is initialized with a base secret seed. This seed initializes a `BLAKE3` hasher.
2.  **Derivation**: When `dkim(selector, domain)` is called, the hasher is cloned and updated with the `selector` and `domain`.
3.  **Key Generation**: The final hash digest is used to seed a **ChaCha20Rng**, which then generates a deterministic **RSA 2048** key pair.
4.  **Output**: The public part of the key is encoded in Base64 (SPKI format) and formatted into a standard DKIM TXT record.

This process ensures that as long as the secret seed remains constant, the generated DKIM keys for any given domain will always be the same.

## API Reference

### `struct Sk`

The main entry point for key generation.

- **`Sk::new(sk: impl AsRef<[u8]>) -> Self`**
  Creates a new `Sk` instance using the provided secret seed.

- **`Sk::dkim(&self, selector: impl AsRef<str>, domain: impl AsRef<str>) -> Dkim`**
  Derives a `Dkim` instance for the specified selector (`selector`) and domain (`domain`).

### `struct Dkim`

Represents the generated DKIM key pair.

- **`pub sk: rsa::RsaPrivateKey`**
  The underlying RSA private key.

- **`Dkim::txt(&self) -> String`**
  _(Requires `pk` feature)_
  Returns the formatted DKIM DNS TXT record string (e.g., `v=DKIM1; k=rsa; p=...`).

## Tech Stack

- **Rust**: Core language.
- **rsa**: Pure Rust implementation of RSA.
- **rand_chacha**: Cryptographically secure random number generator (ChaCha20) used for deterministic key generation from the seed.
- **blake3**: Cryptographic hashing for deterministic seed derivation.
- **base64**: Encoding the public key for DNS records.

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
