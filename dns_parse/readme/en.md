# dns_parse : DNS Message Build and Parse

A lightweight DNS message builder and parser library, used by [idoq](https://crates.io/crates/idoq) (DoQ) and [idot](https://crates.io/crates/idot) (DoT).

## Features

- Build DNS query messages with EDNS support
- Parse DNS response messages
- Support A, AAAA, MX, TXT, NS, CNAME, PTR, SRV record types
- DNS name compression (pointer) handling
- Zero-copy parsing with `bytes` crate

## Installation

```toml
[dependencies]
dns_parse = "0.1"
```

## Usage

### Build DNS Query

```rust
use dns_parse::build;

// Build query with message ID, domain, and query type
let msg = build(0x1234, "example.com", 1); // A record
let msg = build(0, "example.com", 28);     // AAAA record (ID=0 for DoQ)
```

### Parse DNS Response

```rust
use dns_parse::parse;
use bytes::Bytes;

let response: Bytes = /* DNS response data */;
match parse(response) {
  Ok(answers) => {
    for a in answers {
      println!("{} {} TTL={}", a.name, a.val, a.ttl);
    }
  }
  Err(e) => eprintln!("Parse error: {e}"),
}
```

## API Reference

### Functions

#### `build`

Build a DNS query message.

```rust
pub fn build(id: u16, domain: &str, qtype: u16) -> Bytes
```

- `id`: Message ID (0 for DoQ per RFC 9250, random for DoT)
- `domain`: Query domain name
- `qtype`: Query type (1=A, 28=AAAA, 15=MX, etc.)

Returns DNS query message with EDNS OPT record (4096 byte UDP payload).

#### `parse`

Parse a DNS response message.

```rust
pub fn parse(data: Bytes) -> Result<Vec<Answer>>
```

Returns parsed answer records. Empty vector for NXDOMAIN or error responses.

### Types

#### `Answer` (from idns)

```rust
pub struct Answer {
  pub name: String,   // Record name
  pub val: String,    // Record value (formatted)
  pub type_id: u16,   // Record type
  pub ttl: u32,       // Time to live
}
```

### Error Types

- `ResponseTooShort` - Response less than 12 bytes
- `IncompleteData` - Truncated record data
- `NameOutOfBounds` - Name extends beyond message
- `PointerOutOfBounds` - Invalid compression pointer

## Record Value Formats

| Type         | Format                      |
| ------------ | --------------------------- |
| A (1)        | `192.0.2.1`                 |
| AAAA (28)    | `2001:db8::1`               |
| MX (15)      | `10 mail.example.com`       |
| TXT (16)     | `v=spf1 include:...`        |
| NS/CNAME/PTR | `ns1.example.com`           |
| SRV (33)     | `10 5 5060 sip.example.com` |
| Other        | Hex encoded                 |

## Tech Stack

| Component | Library   |
| --------- | --------- |
| Buffer    | bytes     |
| Error     | thiserror |
| Hex       | hex       |
