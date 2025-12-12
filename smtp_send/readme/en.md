# smtp_send : Secure SMTP Email Sending with DKIM

`smtp_send` is a robust Rust library designed for sending emails via SMTP with built-in support for DKIM signing, automatic MX record lookups, and automatic rejection handling. It simplifies the process of sending authenticated emails, ensuring high deliverability and security.

## Table of Contents

- [Introduction](#introduction)
- [Features](#features)
- [Usage](#usage)
- [Design](#design)
- [Technology Stack](#technology-stack)
- [Directory Structure](#directory-structure)
- [API Reference](#api-reference)
- [History](#history)

## Introduction

Sending emails programmatically often involves complex configurations, especially when dealing with authentication standards like DKIM (DomainKeys Identified Mail) and finding the correct mail servers (MX records). `smtp_send` abstracts these complexities, providing a streamlined interface to send signed emails directly to recipient mail servers. It also handles delivery failures by automatically generating and sending rejection reports to the sender.

## Features

- **Automatic DKIM Signing**: Signs emails using RSA-SHA256 to ensure authenticity and integrity.
- **Smart MX Lookup**: Automatically resolves MX records for recipient domains using DNS-over-HTTPS (DoH).
- **Recipient Grouping**: Efficiently groups recipients by domain to minimize connections.
- **Failover Support**: Tries multiple MX servers if the primary one fails.
- **Automatic Rejection Reports**: Sends a detailed rejection email with error logs and original message attachments back to the sender if delivery fails. Uses RFC 5321 compliant null sender to prevent loops.
- **Security Best Practices**: Implements RFC 6376 recommendations.

## Usage

Here is a basic example of how to use `smtp_send` to send an email.

```rust
use smtp_send::Send;
use mail_struct::Mail;
use std::collections::{HashMap, HashSet};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Load DKIM private key from file
    let sk_bytes = std::fs::read("path/to/private.key")?; 

    // 2. Create sender using the new() method
    let sender = Send::new("default", &sk_bytes); // "default" is your DKIM selector

    // 3. Construct the Email
    // Note: In a real application, you might use a helper to construct this
    let mut host_user_li = HashMap::new();
    let mut users = HashSet::new();
    users.insert("recipient".to_string());
    host_user_li.insert("example.com".to_string(), users);

    let mail = Mail {
        sender_user: "sender".to_string(),
        sender_host: "yourdomain.com".to_string(),
        host_user_li,
        body: b"Subject: Test Email\r\n\r\nThis is a test email.".to_vec(),
    };

    // 4. Send the Email
    // Returns a SendResult struct containing success count and error details
    let result = sender.send(&mail).await;

    println!("Successfully sent to {} recipients", result.success);

    for error in result.error_li {
        println!("Error: {:?}", error);
    }

    Ok(())
}
```

> Note: See `tests/main.rs` for more comprehensive examples.

## Design

The library follows a logical flow to ensure reliable delivery:

1.  **Input**: Takes a `Mail` object and DKIM configuration (`Send` struct).
2.  **Grouping**: Recipients are grouped by their domain.
3.  **MX Resolution**: For each domain, the library queries DNS (via DoH) to find the Mail Exchange (MX) records.
4.  **Signing**: The email is cryptographically signed using the provided Private Key and Selector.
5.  **Transmission**: The library connects to the target SMTP server (port 25) and delivers the signed message.
6.  **Rejection Handling**: If delivery fails, a rejection email is automatically constructed (preserving original headers and body) and sent back to the sender with error details, using a null sender (`MAIL FROM:<>`) to prevent infinite loops.
7.  **Result**: Returns a summary of success counts and specific errors.

## Technology Stack

-   **Rust**: The core language, chosen for safety and performance.
-   **mail_send**: Handles the low-level SMTP protocol interactions.
-   **idoh**: Performs DNS-over-HTTPS lookups for MX records.
-   **sk_dkim**: Manages DKIM secret keys and signing operations.
-   **mail-parser**: Parses original emails for rejection report generation.

## Directory Structure

```
.
├── Cargo.toml      # Project configuration and dependencies
├── readme/         # Documentation files
│   ├── en.md       # English README
│   └── zh.md       # Chinese README
├── src/            # Source code
│   ├── dkim.rs     # DKIM signer implementation and caching
│   ├── error.rs    # Error definitions
│   ├── reject/     # Rejection email generation logic
│   ├── send.rs     # SMTP sending logic
│   ├── smtp.rs     # SMTP connection and sending wrapper
│   └── lib.rs      # Main library entry point
└── tests/          # Integration tests
    └── main.rs     # Usage examples and verification tests
```

## API Reference

### `struct Send`

The main configuration struct for sending emails.

-   `selector: String`: The DKIM selector.
-   `sk: Sk`: The Secret Key used for signing.

### `impl Send`

#### `fn new(selector: impl Into<String>, sk: impl AsRef<[u8]>) -> Self`

Creates a new `Send` instance.

-   **Parameters**:
    -   `selector`: The DKIM selector.
    -   `sk`: The DKIM private key bytes.
-   **Returns**:
    -   `Send`: The configured sender instance.

#### `async fn send(&self, mail: &Mail) -> SendResult`

Sends the provided email to all recipients. Automatically handles rejections for failed deliveries.

-   **Parameters**:
    -   `mail`: A reference to the `Mail` struct.
-   **Returns**:
    -   `SendResult`: A struct containing the results.

### `struct SendResult`

The result of a send operation.

-   `error_li: Vec<Error>`: A list of errors encountered during sending.
-   `success: usize`: The number of recipients the email was successfully sent to.

### `enum Error`

-   `DnsResolveFailed(String, idoh::Error)`: DNS resolution failed for a host.
-   `MxIsEmpty(String)`: No MX records found for a host.
-   `Reject(String, smtp_proto::Response<String>)`: Message rejected by the server (for a specific recipient).
-   `SendErr(String, mail_send::Error)`: Failed to send to a recipient.
-   `SmtpAllFailed(String, mail_send::Error)`: Failed to connect to or send via all available MX servers for a domain.

## History

### The Merger that Created DKIM

In the early 2000s, as email spam and phishing became rampant, two major tech giants independently worked on solutions. Yahoo! developed **DomainKeys**, focusing on verifying the DNS domain of a sender. Simultaneously, Cisco created **Identified Internet Mail (IIM)**, which proposed a signature-based authentication standard.

Recognizing that a unified standard would be more effective, these two distinct approaches were merged in 2004. This collaboration birthed **DKIM (DomainKeys Identified Mail)**. It combined the cryptographic integrity of IIM with the domain verification of DomainKeys. This unified specification eventually became an Internet Standard (RFC 6376) in 2011, becoming a cornerstone of modern email security alongside SPF and DMARC.