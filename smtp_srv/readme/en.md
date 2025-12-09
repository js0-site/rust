# smtp_srv : High-performance SMTPS server with auto-refreshing certificates via Redis / Kvrocks

> [!IMPORTANT]
> Configure the following environment variables before deployment. Ensure sensitive keys are protected.

```bash
# DKIM Configuration
DKIM_SK="B3H-XxXXxxXXxXxx"
DKIM_PREFIX="js0-rsa"

# SMTP Authentication (loaded via auth_env)
SMTP_PASSWORD=XxXXXXX
SMTP_USER=i@js0.site
```

## Table of Contents

- [Introduction](#introduction)
- [Features](#features)
- [Architecture](#architecture)
- [Tech Stack](#tech-stack)
- [Directory Structure](#directory-structure)
- [API Reference](#api-reference)
- [Usage & Demo](#usage--demo)
- [Email History: The Rise of Authentication](#email-history-the-rise-of-authentication)

## Introduction

`smtp_srv` is a robust SMTPS server implementation written in Rust, designed for reliability and zero-downtime maintenance. Its standout capability is the integration with [Redis](https://redis.io/) / [Kvrocks](https://kvrocks.apache.org/) for dynamic SSL certificate management. Certificates are automatically loaded and refreshed from the database before expiration, eliminating the need to restart the server when certificates update.

This project serves as the concrete implementation layer, binding the core `smtp_recv` engine with specific storage and authentication strategies.

## Features

*   **Zero-Downtime Certificate Rotation**: Leverages `crate::smtp_recv` to automatically load SSL/TLS certificates from Redis / Kvrocks.
*   **Auto-Refresh**: Monitors certificate validity and fetches fresh certificates seamlessly.
*   **High Performance**: Built on top of the asynchronous `tokio` runtime and efficient `mimalloc` memory allocator.
*   **Secure Defaults**: Enforces SMTPS (465) and integrates strict authentication flows.
*   **DKIM Signing**: Automated DKIM signature injection for outgoing mail reliability.

## Architecture

The system is designed with a modular "Receive-Process-Forward" pipeline:

1.  **Transport Layer**: `smtp_recv` handles the raw SMTP protocol state machine.
2.  **Certificate Provider**: `Cert` struct (implementing `ssl_trait::CertByHost`) delegates to `cert_by_host`. It queries Redis / Kvrocks based on the incoming connection's SNI (Server Name Indication), extracting the Top Level Domain (TLD) if necessary to find the matching certificate.
3.  **Authentication**: `AuthEnv` loads SMTP credentials, securing the relay access.
4.  **Message Handling**: Authenticated messages are passed to `Mailer`.
5.  **Delivery**: `Mailer` uses `smtp_send` to dispatch the email to its final destination, signing it with DKIM keys loaded from environment variables.

## Tech Stack

*   **Runtime**: `tokio` (Async I/O)
*   **Core Engine**: `smtp_recv` (SMTP Protocol implementation)
*   **Storage/Certificates**: `redis / kvrocks`, `cert_by_host`
*   **Cryptography**: `rustls` (Modern, safe TLS library)
*   **Memory Management**: `mimalloc` (High-performance allocator)
*   **Utilities**: `aok` (Error handling), `genv` (Environment parsing)

## Directory Structure

*   `src/lib.rs`: Library entry point, exports core modules.
*   `src/main.rs`: Application entry point, initializes runtime and global allocator.
*   `src/cert.rs`: Implements dynamic certificate retrieval from Redis / Kvrocks.
*   `src/mailer.rs`: Implements the mail delivery logic, connecting reception to transmission.
*   `test/`: Contains integration tests demonstrating usage with Node.js clients (`nodemailer`).

## API Reference

The library exposes the following key components:

### `Cert`
A zero-sized struct implementing `ssl_trait::CertByHost`.
*   **Functionality**: Intercepts TLS handshake requests, parses the hostname/TLD, and retrieves the corresponding active certificate from Redis / Kvrocks.

### `Mailer`
The email processing agent implementing `smtp_recv::Mailer`.
*   **Functionality**: Receives `UserMail` objects (containing the authenticated user ID and raw email content) and forwards them using the configured `smtp_send` transport.

### `run(port: u16) -> Void`
The main server loop.
*   **Signature**: `async fn run(port: u16) -> Void`
*   **Usage**: Starts the SMTPS server on the specified port, injecting the `AuthEnv`, `Mailer`, and `Cert` providers.

## Usage & Demo

See `@tests/` for a complete example using `nodemailer`.

To run the server locally:
```bash
# Ensure environment variables are set (see top of document)
cargo run --release
```

Test script excerpt (`test/test_smtp.js`):
```javascript
const SMTP = nodemailer.createTransport({
  host: "127.0.0.1",
  port: 465,
  secure: true, // Uses SMTPS
  auth: { user: SMTP_USER, pass: SMTP_PASSWORD },
  tls: { servername: "smtp.js0.site" }, // Trigger SNI for cert loading
});
```

## Email History: The Rise of Authentication

In the early days of ARPANET, email was a trusting system. Protocol designers in the 1980s didn't anticipate the modern era of spam and spoofing. By the early 2000s, this trust was broken. 

The response was a fragmented evolution of authentication standards. **DomainKeys** (from Yahoo!) and **Identified Internet Mail** (from Cisco) were competing approaches to verify sender identity. In 2004, these giants merged their efforts, creating **DKIM (DomainKeys Identified Mail)**.

DKIM brought cryptographic signatures to email headers, allowing receiving servers to verify that the email truly originated from the claimed domain and hadn't been tampered with. It was a pivotal moment, shifting email from a "best effort" delivery system to a verifiable trust network, laying the groundwork for the modern DMARC policies that protect inboxes today. This server implements these standards to ensure your transactional emails are trusted by recipients.