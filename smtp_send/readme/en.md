# smtp_send

SMTP email sending library with DKIM signing, MX lookup, and automatic bounce handling.

## Features

- **DKIM Signing**: RSA-SHA256 with RFC 6376 header protection (N+1 signing)
- **MX Lookup**: DNS-over-QUIC via `idot`
- **Concurrent Sending**: Parallel delivery grouped by domain
- **Failover**: Tries multiple MX servers on failure
- **Bounce Handling**: Auto-generates RFC 5321 compliant rejection reports
- **Loop Prevention**: Checks `Received` header count (max 30) and `Auto-Submitted`/`Return-Path` headers
- **DKIM Cache**: 600s TTL cache for signers

## Usage

```rust
use smtp_send::Send;
use mail_struct::Mail;

#[tokio::main]
async fn main() {
    // Load DKIM private key
    let sk = std::fs::read("private.key").unwrap();

    // Create sender with DKIM selector
    let sender = Send::new("default", &sk);

    // Build email
    let mut mail = Mail::new(
        "sender@example.com",
        ["recipient@example.com"],
        b"Subject: Test\r\n\r\nHello".to_vec(),
    ).unwrap();

    // Send email
    let result = sender.send(&mut mail).await;

    println!("sent: {}, errors: {}", result.success, result.error_li.len());
}
```

## API

### `Send`

```rust
pub struct Send {
    pub selector: String,  // DKIM selector
    pub sk: Sk,            // DKIM private key
}

impl Send {
    // Create sender
    fn new(selector: impl Into<String>, sk: impl AsRef<[u8]>) -> Self;

    // Send email with Received header and DKIM signing
    async fn send(&self, mail: &mut Mail) -> SendResult;
}
```

### `SendResult`

```rust
pub struct SendResult {
    pub success: usize,        // Number of successful deliveries
    pub error_li: Vec<Error>,  // List of errors
}
```

### `Error`

| Variant                       | Description                       |
| ----------------------------- | --------------------------------- |
| `DkimInit(host)`              | DKIM signer initialization failed |
| `DnsResolveFailed(host, err)` | DNS lookup failed                 |
| `MxIsEmpty(host)`             | No MX records found               |
| `Reject(Reject)`              | Server rejected the message       |
| `SendErr(SendErr)`            | Send failed                       |
| `SmtpAllFailed(host, err)`    | All MX servers failed             |
| `TooManyReceived(sender)`     | Exceeded 30 Received headers      |

### Standalone Functions

```rust
// Send without Received header (for direct sending)
pub async fn send(mail: &Mail, signer: Option<&Signer>) -> SendResult;

// Create DKIM signer (cached), returns None on failure
pub fn signer(selector: &str, host: &str, sk: &Sk) -> Option<RefVal<String, Signer>>;

// Check if Received headers exceed limit
pub fn recv_overflow(body: &[u8]) -> bool;

// Add Received header
pub fn add_received(body: &mut Vec<u8>, from: &str, by: &str);

pub const MAX_RECEIVED: usize = 30;
```

## Directory Structure

```
src/
├── lib.rs      # Entry point, Send struct, concurrent send
├── dkim.rs     # DKIM signer with cache
├── error.rs    # Error types
├── parse.rs    # Received header parsing
├── send.rs     # Single domain sending logic
├── smtp.rs     # SMTP connection wrapper
└── reject/     # Bounce email generation
    ├── mod.rs
    ├── create_tar_zstd.rs
    ├── encode_mail.rs
    └── reject_mail.rs
```

## Tech Stack

- `mail_send` - SMTP protocol
- `idot` / `idns` - DNS-over-QUIC for MX lookup
- `sk_dkim` - DKIM key management
- `mail-parser` - Email parsing for bounce generation
- `papaya` - DKIM signer caching
- `async-scoped` - Scoped async tasks for concurrent sending
