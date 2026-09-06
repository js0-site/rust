# idoq : DNS over QUIC Client for Rust

Based on [idns](https://crates.io/crates/idns). See idns for `DnsRace`, `Cache`, `Parse` trait, and more.

## Features

- RFC 9250 compliant DoQ implementation
- Built-in DoQ server list (AdGuard, ControlD, Alibaba DNS)
- Async/await with Tokio
- TLS 1.3 over QUIC
- A, AAAA, MX, TXT, NS, CNAME, PTR, SRV record types
- Connection reuse with auto-reconnect
- 9s timeout

## Installation

```toml
[dependencies]
idoq = "0.1"
idns = "0.1"
```

## Usage

### DnsRace + Cache (Recommended)

Race multiple servers and cache results:

```rust
use idoq::{DOQ_LI, doq_li};
use idns::{Cache, DnsRace, Mx, Query};
use std::time::Instant;

#[tokio::main]
async fn main() {
  let race = DnsRace::new(doq_li(DOQ_LI));
  let cache: Cache<Mx> = Cache::new(60); // 60s TTL

  // First query (cache miss)
  let t1 = Instant::now();
  let r1 = cache.query(&race, "gmail.com").await;
  let d1 = t1.elapsed();
  println!("First: {}ms", d1.as_millis());
  if let Some(mx_list) = &*r1.unwrap() {
    for mx in mx_list {
      println!("  {} {}", mx.priority, mx.server);
    }
  }

  // Second query (cache hit)
  let t2 = Instant::now();
  let _ = cache.query(&race, "gmail.com").await;
  let d2 = t2.elapsed();
  println!("Cache: {}μs", d2.as_micros());
  println!("✓ {}ms -> {}μs ({}x faster)", d1.as_millis(), d2.as_micros(),
    d1.as_micros() / d2.as_micros().max(1));
}
```

Output:

```
First: 22ms
  5 gmail-smtp-in.l.google.com
  10 alt1.gmail-smtp-in.l.google.com
  20 alt2.gmail-smtp-in.l.google.com
  30 alt3.gmail-smtp-in.l.google.com
  40 alt4.gmail-smtp-in.l.google.com
Cache: 0μs
✓ 22ms -> 0μs (22024x faster)
```

### Basic Query

```rust
use idoq::{Doq, host_ip, QType};
use idns::Query;

#[tokio::main]
async fn main() {
  let client = Doq::new(host_ip("dns.alidns.com", 223, 5, 5, 5));

  if let Ok(Some(answers)) = client.answer_li(QType::A, "example.com").await {
    for a in answers {
      println!("{} TTL={}", a.val, a.ttl);
    }
  }
}
```

## API Reference

### Structs

#### `Doq`

DoQ client with connection reuse. Implements `idns::Query` trait.

#### `HostIp`

Server configuration with `host: SmolStr` (TLS SNI) and `ip: IpAddr`.

### Functions

- `host_ip(host, a, b, c, d) -> HostIp` - Create HostIp from hostname and IPv4
- `doq_li(li: &[HostIp]) -> Vec<Doq>` - Create Doq clients from HostIp list

### Constants

#### `DOQ_LI`

Pre-configured DoQ servers:

| Server      | IP                           |
| ----------- | ---------------------------- |
| AdGuard DNS | 94.140.14.140, 94.140.14.141 |
| ControlD    | 76.76.2.11                   |
| Alibaba DNS | 223.5.5.5, 223.6.6.6         |

## Architecture

```mermaid
graph TD
    A[Client] --> B[Doq.query]
    B --> C[conn]
    C --> D{Alive?}
    D -->|Yes| E[Reuse]
    D -->|No| F[dial]
    F --> G[QUIC + TLS 1.3]
    G --> I[Connection]
    E --> J[send]
    I --> J
    J --> K[DNS Message]
    K --> L[Response]
    L --> M[Parse]
    M --> N[Answers]
```

### Implementation Details

- DNS message ID = 0 (RFC 9250)
- 2-byte length prefix for framing
- EDNS OPT with 4096 byte payload
- `LazyLock` for TLS `ClientConfig`
- `RwLock<Option<Connection>>` for connection reuse
- Auto-reconnect on error
- 9s timeout

## Tech Stack

| Component | Library       |
| --------- | ------------- |
| QUIC      | quinn         |
| TLS       | rustls + ring |
| Async     | tokio         |
| Buffer    | bytes         |
| Error     | thiserror     |
| DNS Parse | dns_parse     |
