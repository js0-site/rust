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

- **Core Structure**: Defines `Mail` and `UserMail` structs to represent email data.
- **Serialization**: Optional `encode` and `decode` features using `bitcode` for high-performance binary serialization.
- **SMTP Integration**: Optional `send` feature with domain-based recipient grouping for efficient email delivery.
- **Type Safety**: Leverages Rust's type system to ensure data integrity.

## Usage

Add `mail_struct` to your `Cargo.toml`:

```toml
[dependencies]
mail_struct = { version = "0.1.7", features = ["send", "encode", "decode"] }
```

### Creating and Grouping Emails by Domain

```rust
use mail_struct::Mail;

#[cfg(feature = "send")]
async fn example() {
    let mail = Mail {
        sender: "sender@example.com".to_string(),
        to_li: vec![
            "user1@gmail.com".to_string(),
            "user2@yahoo.com".to_string(),
            "user3@gmail.com".to_string(),
        ],
        body: b"Hello, this is a test email!".to_vec(),
    };

    // With the 'send' feature enabled, group recipients by domain
    let domain_mail = mail.domain_mail();
    
    for item in domain_mail {
        println!("Sending to domain: {}", item.domain);
        // item.mail is a mail_send::smtp::message::Message
        // client.send(item.mail).await?;
    }
}
```

## Design Philosophy

The library follows a separation of concerns principle. The core `lib.rs` defines the data structures (`Mail`, `UserMail`), keeping the base dependency footprint minimal. Functionalities like serialization and sending are gated behind feature flags (`encode`, `decode`, `send`), allowing users to opt-in only for what they need.

When the `send` feature is active, the `send.rs` module provides the `domain_mail` method that groups recipients by their email domain. This optimization reduces the number of SMTP connections needed and improves delivery efficiency, especially when sending to multiple recipients across different domains.

## Tech Stack

- **Rust**: Core language.
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
│   └── send.rs         # Domain grouping and SMTP message logic (feature: send)
└── tests
    └── main.rs         # Integration tests
```

## API Documentation

### `struct Mail`

Represents a basic email message.

- `sender: String`: The email address of the sender.
- `to_li: Vec<String>`: A list of recipient email addresses.
- `body: Vec<u8>`: The raw body content of the email.

#### Methods

##### `domain_mail<'a>(&'a self) -> Vec<DomainMail<'a>>` (requires `send` feature)

Groups recipients by their email domain and creates a `DomainMail` for each domain. This allows for efficient batch sending where multiple recipients in the same domain can be delivered in a single SMTP transaction.

### `struct UserMail`

A wrapper around `Mail` associating it with a user ID.

- `mail: Mail`: The email content.
- `user_id: u64`: The unique identifier of the user associated with this mail.

### `struct DomainMail<'a>` (requires `send` feature)

Represents an email grouped by recipient domain.

- `domain: &'a str`: The domain name (e.g., "gmail.com").
- `mail: Message<'a>`: The ready-to-send SMTP message containing all recipients for this domain.

## Historical Context

**RFC 822 and the Separation of Envelope and Content**

The design of email systems dates back to the early 1980s with the publication of **RFC 822** (Standard for the Format of ARPA Internet Text Messages) and **RFC 821** (Simple Mail Transfer Protocol). A key architectural decision was the separation of the "envelope" (handled by SMTP for routing) from the "content" (the message headers and body defined by RFC 822).

`mail_struct` honors this tradition by focusing on the *structure* of the message (the content), while delegating the *transport* (the envelope and transmission) to specialized libraries like `mail-send`. This modular approach mirrors the original design philosophy of the internet's most enduring communication protocol, ensuring flexibility and maintainability.