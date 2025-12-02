# msgq : Robust Redis Stream Message Queue

Robust Redis Stream based message queue with auto-claim and retry handling.

## Table of Contents

- [Features](#features)
- [Usage](#usage)
- [Design](#design)
- [Tech Stack](#tech-stack)
- [Directory Structure](#directory-structure)
- [API Reference](#api-reference)
- [History](#history)

## Features

- **Redis Stream Based**: Utilizes `XREADGROUP` for efficient, scalable message consumption.
- **Consumer Groups**: Supports multiple consumers within a group for parallel processing.
- **Automatic Group Creation**: Automatically creates the Redis Stream group and consumer if they don't exist.
- **Reliable Delivery**: Implements auto-claiming of idle pending messages to prevent message loss.
- **Concurrent Processing**: Uses `tokio::spawn` to process messages concurrently for high throughput.
- **Configurable Retries**: Automatic retry mechanism for failed messages, with a configurable limit.
- **Centralized Configuration**: A simple `Conf` struct to manage all connection and behavior settings.
- **Trait-Based Callbacks**: Uses a `Parse` trait for clear and reusable message processing and error handling logic.

## Usage

Define a struct and implement the `Parse` trait to handle your message processing and error logic.

```rust
use msgq::{Conf, Kv, Parse, ReadGroup};
use std::future::Future;
use std::sync::Arc;
use aok::{OK, Void};
use log::info;

// 1. Define your message processor
#[derive(Clone)]
struct MyParser;

impl Parse for MyParser {
  // 2. Implement the message processing logic
  async fn run(&self, kv: &Kv) -> Void {
    info!("run: {:?}", kv);
    OK
  }

  // 3. Implement the error handling logic for messages that fail all retries
  async fn on_error(&self, kv: Kv, err: String) -> Void {
    info!("on_error: {:?}, err: {}", kv, err);
    OK
  }
}

#[tokio::main]
async fn main() -> Void {
  // 4. Initialize the environment (e.g. using xboot to set up global Redis client)
  // xboot::init().await?; 

  // 5. Configure the consumer
  let conf = Conf::new(
    "s1", // stream key
    "g1", // group name
    "c1", // consumer name
    5,    // block_sec: wait up to 5s for new messages
    60,   // claim_idle_sec: claim messages idle for 60s
    10,   // count: batch size
    3,    // max_retry: retry 3 times before on_error
  );

  // 6. Create a ReadGroup and run it
  ReadGroup::new(MyParser, conf).run().await?;
  OK
}
```

## Design

The `ReadGroup::run` method executes a continuous loop that ensures robust message processing:

1.  **Claim Idle Messages**: It first calls `XPENDING` to find messages that have been idle for longer than `claim_idle_ms` and claims them using `XAUTOCLAIM`. This ensures that messages from crashed or slow consumers are re-processed.
2.  **Fetch New Messages**: It then executes `XREADGROUP` with a `BLOCK` timeout to efficiently wait for and receive a new batch of messages.
3.  **Group Management**: If the command fails with a `NOGROUP` error, the `auto_new` function is called to automatically create the consumer group, making setup seamless.
4.  **Parse and Process**: All claimed and new messages are parsed by `parse_stream` into a list of `StreamItem`s.
5.  **Concurrent Execution**: For each `StreamItem`, a `tokio` task is spawned. Inside the task, the `run` method of the provided `Parse` trait implementation is called to execute the user-defined logic.
6.  **Error Handling & Retry**:
    -   If the `run` method returns an error, the system will allow the message to be re-claimed and retried later.
    -   The `retry` count for each message is tracked. If a message's retry count exceeds `max_retry`, it is passed to the `on_error` callback of the `Parse` trait for final handling (e.g., moving to a dead-letter queue).
7.  **Cleanup**: Successfully processed messages (or those handled by `on_error`) are acknowledged and deleted from the stream using `rm_id_li` (`XACK` and `XDEL`) to prevent reprocessing.

## Tech Stack

-   **Rust**: Core language for performance and safety.
-   **Tokio**: Asynchronous runtime for handling concurrency.
-   **Fred**: A high-performance, low-level Redis client for Rust.
-   **ThisError**: A library for deriving boilerplate `Error` implementations.

## Directory Structure

-   `src/lib.rs`: The library's main entry point. It exports the public API, including the `Parse` trait and key structs like `Conf`, `ReadGroup`, and `StreamItem`.
-   `src/conf.rs`: Defines the `Conf` struct, which centralizes all configuration parameters.
-   `src/read_group.rs`: Contains the core consumer logic within the `ReadGroup` struct and its `run` method.
-   `src/auto_new.rs`: Provides the `auto_new` function to automatically create a stream consumer group.
-   `src/parse_stream.rs`: Includes utilities for parsing responses from `XREADGROUP` and `XAUTOCLAIM`.
-   `src/rm_id_li.rs`: A helper function to `XACK` (acknowledge) and `XDEL` (delete) processed messages.
-   `src/error.rs`: Defines custom error types for the application.
-   `tests/`: Integration tests demonstrating usage patterns.

## API Reference

### `Conf`

A struct to hold all configuration for the `ReadGroup` consumer.

-   `stream`: The Redis key for the stream.
-   `group`: The consumer group name.
-   `consumer`: A unique name for this consumer.
-   `block_ms`: The time in milliseconds to block waiting for new messages.
-   `claim_idle_ms`: The idle time in milliseconds after which a pending message is considered abandoned and can be claimed by another consumer.
-   `count`: The maximum number of messages to fetch in a single batch.
-   `max_retry`: The maximum number of times a message will be retried before being passed to the `on_error` handler.

### `Parse` Trait

A trait that defines the application logic for message handling. You must implement this trait.

-   `run(&self, kv: &Kv) -> impl Future<Output = aok::Void> + Send`:
    -   The asynchronous method called to process a single message.
    -   `kv`: The message data, as a `Vec<(Bytes, Bytes)>`.
    -   Return `OK` on success or `Err` on failure. A failed message will be retried later.
-   `on_error(&self, kv: Kv, err: String) -> impl Future<Output = aok::Void> + Send`:
    -   The asynchronous method called when a message has failed more than `max_retry` times.
    -   `kv`: The message data that failed.
    -   `err`: The last error message that caused the failure.

### `ReadGroup`

The main consumer struct.

-   `ReadGroup::new(parse: P, conf: Conf)`: Creates a new `ReadGroup` instance.
-   `run(&self)`: Starts the infinite processing loop.

### `StreamItem`

Represents a single message from the Redis Stream.

-   `id`: The unique ID of the message.
-   `retry`: The delivery count (number of times delivered).
-   `idle_ms`: Time in milliseconds the message has been idle (if claimed).
-   `kv`: The message payload as a vector of Key-Value byte pairs.

## History

In 1983, Vivek Ranadive, a 26-year-old MIT graduate, observed that while hardware components communicated via a "bus", software lacked a similar standard mechanism. He envisioned a "software bus" where applications could publish and subscribe to information without direct, rigid connections. This idea led to the creation of **The Information Bus (TIB)**, the first commercial message queue software. TIB revolutionised financial trading floors by replacing manual chalkboards with real-time digital streams, allowing different trading systems to communicate instantly. This innovation laid the groundwork for modern event-driven architectures and the message queue systems we rely on today, such as Redis Streams.