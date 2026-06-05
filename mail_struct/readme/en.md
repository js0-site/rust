# mail_struct : Minimalist Email Structure for Rust

`mail_struct` is a lightweight Rust library designed to define a clear and efficient structure for email messages. It provides optional integration with `bitcode` for efficient encoding/decoding and `mail-send` for SMTP transmission with domain-based grouping, making it a versatile choice for email handling in Rust applications.

## Table of Contents

- [Features](#features)
- [Usage](#usage)
- [Design Philosophy](#design-philosophy)
- [Tech Stack](#tech-stack)
- [Directory Structure](#directory-structure)
- [API Documentation](#api-documentation)
- [Historical Context](#historical-context)

## Features

- **Core Structure**: Defines `Mail`, `UserMail`, and `HostUserLi` structs to represent email data with type-safe domain grouping.
- **Serialization**: Optional `encode` and `decode` features using `bitcode` for high-performance binary serialization.
- **SMTP Integration**: Optional `send` feature with domain-based recipient grouping for efficient email delivery.
- **Type Safety**: Leverages Rust's type system with custom types like `HostUserLi` to ensure data integrity and prevent misuse.

## Usage

Add `mail_struct` to your `Cargo.toml`:

```toml
[dependencies]
mail_struct = { version = "0.1.18", features = ["send", "encode", "decode"] }
```

### Creating and Grouping Emails by Domain

```rust
use mail_struct::Mail;

#[cfg(feature = "send")]
async fn example() {
    // Mail::new() automatically:
    // - Normalizes and validates email addresses using xmail::norm_user_host
    // - Filters out invalid email addresses
    // - Deduplicates recipients
    // Returns Option<Mail>, None if no valid recipients
    let mail = Mail::new(
        "sender@example.com",
        vec![
            "user1@gmail.com",
            "user2@yahoo.com",
            "user3@gmail.com",
            "user1@gmail.com",  // Duplicate - will be removed
            "invalid-email",     // Invalid - will be filtered out
        ],
        b"Hello, this is a test email!",
    ).unwrap(); // Handle None in production

    // Directly iterate over &mail to get MailMessage grouped by domain
    for msg in &mail {
        println!("Sending to domain: {}", msg.domain);

        // 1. Try batch sending to all recipients in this domain
        // MailMessage implements IntoMessage trait, can be passed directly to client.send()
        if let Err(e) = client.send(&msg).await {
            println!("✗ Batch send failed: {}, sending individually", e);

            // 2. If batch send fails, use IntoIterator to send individually
            // This uses a custom zero-allocation iterator
            for individual_message in msg {
                match client.send(individual_message).await {
                    Ok(_) => println!("  ✓ Individual send successful"),
                    Err(e) => println!("  ✗ Individual send failed: {}", e),
                }
            }
        } else {
            println!("✓ Batch send successful");
        }
    }
}
```

## Design Philosophy

The library follows a separation of concerns principle. The core `lib.rs` defines the data structures (`Mail`, `UserMail`), keeping the base dependency footprint minimal. Functionalities like serialization and sending are gated behind feature flags (`encode`, `decode`, `send`), allowing users to opt-in only for what they need.

When the `send` feature is active, the `send.rs` module implements `IntoIterator` for `&Mail`, returning `MailMessage` instances grouped by domain. This optimization reduces the number of SMTP connections needed and improves delivery efficiency. Additionally, `MailMessage` also implements `IntoIterator`, providing a zero-overhead fallback mechanism that ensures individual delivery to valid recipients when batch sending fails due to invalid recipient addresses.

## Tech Stack

- **Rust**: Core language.
- **xmail**: Email validation and normalization.
- **bitcode** (Optional): For fast binary encoding and decoding.
- **mail-send** (Optional): For SMTP message construction and sending.

## Directory Structure

```
.
├── Cargo.toml          # Project configuration
├── README.md           # Main documentation
├── readme              # Documentation in specific languages
│   ├── en.md           # English README
│   └── zh.md           # Chinese README
├── src
│   ├── lib.rs          # Core struct definitions and feature gates
│   ├── host_user_li.rs # HostUserLi type for domain-based recipient grouping
│   └── send.rs         # SMTP message logic (feature: send)
└── tests
    └── main.rs         # Integration tests
```

## API Documentation

### `struct Mail`

Represents a basic email message.

- `sender_user: String`: The user part of the sender's email address.
- `sender_host: String`: The domain part of the sender's email address.
- `host_user_li: HostUserLi`: Recipients grouped by domain using a type-safe wrapper around `HashMap<String, HashSet<String>>`.
- `body: Vec<u8>`: The raw body content of the email.

### `struct HostUserLi`

A type-safe wrapper for domain-based recipient grouping that implements `Deref<Target = HashMap<String, HashSet<String>>>`.

#### Methods

##### `add(&mut self, mail: impl AsRef<str>) -> bool`

Adds an email address with automatic validation and normalization using `xmail::norm_user_host`. Returns `true` if the email was valid and added successfully, `false` otherwise. Invalid emails are logged and ignored.

##### `user_li(&self, host: &str) -> Option<&HashSet<String>>`

Returns the set of users for a given host domain, or `None` if the domain doesn't exist.

#### Trait Implementations

##### `Deref<Target = HashMap<String, HashSet<String>>>`

Allows direct read-only access to underlying HashMap methods like `len()`, `is_empty()`, `iter()`, `get()`, etc.

##### `DerefMut<Target = HashMap<String, HashSet<String>>>`

Provides mutable access to the underlying HashMap, enabling direct manipulation using methods like `entry()`, `insert()`, `remove()`, etc. This allows for advanced operations while maintaining type safety.

##### `FromIterator<T: AsRef<str>>`

Enables creation from any iterator of string-like items using `from_iter()`. Each item is validated and added automatically, making it easy to create `HostUserLi` from collections of email addresses.

#### Methods

##### `new(sender: impl AsRef<str>, to_li: impl IntoIterator<Item = impl AsRef<str>>, body: impl Into<Vec<u8>>) -> Option<Self>`

Creates a new `Mail` instance with automatic email processing:

- **Sender Validation**: Uses `xmail::norm_user_host` to validate and normalize the sender address.
- **Recipient Processing**: Uses `HostUserLi::from_iter()` to process all recipients in a functional style.
- **Automatic Validation**: Each recipient is validated and normalized using `xmail::norm_user_host`.
- **Filtering**: Invalid email addresses are automatically filtered out and logged.
- **Deduplication**: Recipients are automatically deduplicated using `HashSet` within each domain.
- **Grouping**: Recipients are organized by domain (host) for efficient processing.
- **Return**: Returns `None` if no valid recipients remain after filtering, or if the sender is invalid.

#### Trait Implementations

##### `IntoIterator` for `&'a Mail` (requires `send` feature)

Implements `IntoIterator` for `&Mail`, returning a custom `MailIter<'a>` iterator.

- **Domain Grouping**: Automatically groups recipients by email domain, with each `MailMessage` containing all recipients for the same domain.
- **Lazy Construction**: The iterator lazily creates `MailMessage` instances during iteration, avoiding pre-allocation of a `Vec`.
- **Efficient Delivery**: Multiple recipients in the same domain can be delivered in a single SMTP transaction.

### `struct UserMail`

A wrapper around `Mail` associating it with a user ID.

- `mail: Mail`: The email content.
- `user_id: u64`: The unique identifier of the user associated with this mail.

### `struct MailMessage<'a>` (requires `send` feature)

Represents an email grouped by recipient domain.

- `sender_user: &'a str`: The user part of the sender's email address.
- `sender_host: &'a str`: The domain part of the sender's email address.
- `domain: &'a str`: The recipient domain name (e.g., "gmail.com").
- `to_li: Vec<Address<'a>>`: List of all recipient addresses for this domain.
- `body: &'a [u8]`: The email body content.

#### Trait Implementations

##### `IntoMessage<'a>`

Implements the `mail_send::smtp::message::IntoMessage` trait, allowing `MailMessage` to be directly converted into an SMTP message containing all recipients. This enables `MailMessage` to be passed directly to `client.send()` for batch sending.

Also implements `IntoMessage` for `&MailMessage`, allowing message creation without consuming ownership.

##### `IntoIterator`

Implements the `IntoIterator` trait, returning a custom `MailMessageIter` iterator.

- **Zero Allocation**: The iterator lazily constructs `Message` instances during iteration, avoiding the overhead of creating an intermediate `Vec`.
- **Fallback Strategy**: Converts `MailMessage` into multiple individual `Message` instances (each containing one recipient). This is useful when batch sending fails, allowing for individual delivery to ensure valid recipients receive the email.

## Historical Context

**RFC 822 and the Separation of Envelope and Content**

The design of email systems dates back to the early 1980s with the publication of **RFC 822** (Standard for the Format of ARPA Internet Text Messages) and **RFC 821** (Simple Mail Transfer Protocol). A key architectural decision was the separation of the "envelope" (handled by SMTP for routing) from the "content" (the message headers and body defined by RFC 822).

`mail_struct` honors this tradition by focusing on the _structure_ of the message (the content), while delegating the _transport_ (the envelope and transmission) to specialized libraries like `mail-send`. This modular approach mirrors the original design philosophy of the internet's most enduring communication protocol, ensuring flexibility and maintainability.
