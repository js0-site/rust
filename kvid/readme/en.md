# kvid : Global Unique ID Generator Based on Redis/Kvrocks

- [Introduction](#introduction)
- [Usage](#usage)
- [Design and Implementation](#design-and-implementation)
- [Tech Stack](#tech-stack)
- [Directory Structure](#directory-structure)
- [Competitors](#competitors)
- [Comparison of ID Generation Algorithms](#comparison-of-id-generation-algorithms)
- [History](#history)

## Introduction

`kvid` is a distributed unique ID generator based on Redis or Kvrocks. It guarantees globally unique IDs that are trend-increasing. It is designed to be robust, high-performance, and easy to integrate into Rust projects.

Key features include:
- **Global Uniqueness**: Ensures no duplicate IDs are generated across the distributed system.
- **Trend Increasing**: IDs are generated in an increasing order, which is beneficial for database indexing.
- **High Performance**: Utilizes batch fetching (step-based) to minimize network round-trips to Redis/Kvrocks.
- **Dynamic Step Adjustment**: Automatically adjusts the batch size based on consumption rate to balance performance and ID continuity.
- **Static Global Variable Support**: Can be directly declared as a `static` global variable in Rust, simplifying usage across the application.

## Usage

Add `kvid` to `Cargo.toml`.

### Basic Example

`kvid` allows declaring the generator as a static global variable, making it accessible throughout the application without passing instances around.

```rust
use std::time::Duration;
use aok::{OK, Void};
use kvid::KvId;
use log::info;

// Initialize logger (optional, depending on your setup)
#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
}

// Declare as a static global variable
pub static KVID_TEST: KvId = KvId::new("test");

#[tokio::test]
async fn test() -> Void {
  // Initialize global Redis/Kvrocks connection (Required for xkv to connect to Redis/Kvrocks)
  xboot::init().await?;

  for i in 0..300 {
    // Generate next ID
    let id = KVID_TEST.next().await?;
    info!("{}", id);

    if i > 5 {
      tokio::time::sleep(Duration::from_secs(1)).await;
    }
  }
  OK
}
```

## Design and Implementation

### Configuration & Constants

The following constants are defined in `src/lib.rs` and control the behavior of the generator:

-   **`FETCH_DURATION` (Default: 600s / 10 minutes)**:
    The target duration for a batch of IDs to last. The algorithm tries to adjust the `step` (batch size) so that a fetch request happens approximately every 10 minutes.
    -   If IDs are consumed **faster** than this duration, the step size **doubles** (up to `STEP_MAX`) to reduce network frequency.
    -   If IDs are consumed **slower** than this duration, the step size **halves** to prevent holding too many unused IDs during low traffic periods.

-   **`STEP_MAX` (Default: 1,000,000)**:
    The maximum number of IDs that can be fetched in a single request. This prevents the step size from growing indefinitely.

### Core Logic

The core logic resides in the `KvId` struct. It maintains a local range of IDs and fetches a new range (step) from the Redis/Kvrocks backend when the local range is exhausted.

1.  **Initialization**: `KvId` is initialized with a name (key).
2.  **ID Generation (`next`)**:
    - Checks if there are available IDs in the local buffer (`id < max`).
    - If yes, increments the local `id` and returns it.
    - If no, it triggers a fetch operation.
3.  **Fetching from Backend**:
    - Uses `HINCRBY` command on Redis/Kvrocks to atomically increment the maximum ID for the given key by `step`.
    - Updates local `max` and `id` based on the response.
4.  **Dynamic Step Adjustment**:
    - The system calculates the time elapsed (`cost`) since the last fetch.
    - **Increase Step**: If `cost <= FETCH_DURATION` (high load), `step = step * 2`.
    - **Decrease Step**: If `cost > FETCH_DURATION` (low load), `step = max(1, step / 2)`.
    - This self-tuning mechanism ensures high performance under load while minimizing waste during idle times.

### Data Structures (`lib.rs`)

-   **`KvId`**: The main struct exposed to the user.
    -   `name`: The key name used in Redis.
    -   `inner`: A `Mutex` protected `Inner` state.
-   **`Inner`**: Internal state of the generator.
    -   `id`: Current ID available for distribution.
    -   `max`: The maximum ID in the current allocated range.
    -   `step`: Current batch size to fetch from backend.
    -   `ts`: Timestamp of the last fetch.

## Tech Stack

-   **Rust**: Core language.
-   **Redis / Kvrocks**: Backend storage for atomic counters.
-   **fred**: Async Redis client.
-   **parking_lot**: Efficient Mutex implementation.
-   **tokio**: Async runtime.

## Directory Structure

```
.
├── Cargo.toml      # Project configuration
├── README.mdt      # README template
├── readme/         # Documentation files
│   ├── en.md       # English README
│   └── zh.md       # Chinese README
├── src/            # Source code
│   └── lib.rs      # Library entry point
└── tests/          # Integration tests
    └── main.rs     # Usage demonstration
```

## Competitors

Distributed ID generation is a common requirement. Here are some similar projects:

-   **Baidu Uidgenerator**: Java-based, Snowflake algorithm variant. High performance but relies on Snowflake's time-dependency.
-   **Meituan Leaf**: Supports both Segment mode (database) and Snowflake mode (ZooKeeper). Segment mode is similar to `kvid`'s approach.
-   **Didi TinyID**: Java-based, Segment mode only. Focuses on high availability and multi-db support.

`kvid` distinguishes itself by being written in Rust, offering high performance with low footprint, and specifically optimizing for ease of use with static global variables.

## Comparison of ID Generation Algorithms

To better understand `kvid`'s position, here is a comparison of common distributed ID generation algorithms:

### 1. UUID (Universally Unique Identifier)
-   **Principle**: 128-bit identifier generated based on timestamp, random numbers, or MAC address.
-   **Pros**:
    -   Simple to implement, no network interaction needed.
    -   Globally unique without coordination.
-   **Cons**:
    -   **Too long**: 128 bits (32 hex chars) is inefficient for storage and indexing.
    -   **Not sortable**: Randomness (v4) causes page splitting in B+Tree indexes, hurting database performance.
    -   **Information leakage**: v1 contains MAC address.

### 2. Database Auto-increment
-   **Principle**: Rely on database's `AUTO_INCREMENT` feature.
-   **Pros**:
    -   Simple, strictly increasing.
-   **Cons**:
    -   **Single point of failure**: Database becomes the bottleneck.
    -   **Hard to scale**: Difficult to merge data from multiple databases later.

### 3. Snowflake (Twitter)
-   **Principle**: 64-bit integer: 1 bit sign + 41 bits timestamp + 10 bits machine ID + 12 bits sequence.
-   **Pros**:
    -   High performance (millions of IDs/sec).
    -   Time-ordered (roughly).
    -   No network overhead (generated locally).
-   **Cons**:
    -   **Clock dependency**: Strongly relies on system clock. Clock rollback can cause duplicate IDs or service unavailability.
    -   **Machine ID management**: Requires a mechanism (like ZooKeeper) to assign unique machine IDs.

### 4. Segment Mode (kvid / Meituan Leaf)
-   **Principle**: Pre-allocate a range (step) of IDs from a central store (Redis/DB) and issue them from memory.
-   **Pros**:
    -   **High Performance**: Database is accessed only once per step (e.g., every 1000 IDs).
    -   **No Clock Dependency**: Immune to clock rollback issues.
    -   **Trend Increasing**: Friendly to database indexing.
-   **Cons**:
    -   **ID Gaps**: If the service restarts, unused IDs in the current step are lost (but uniqueness is preserved).
    -   **Central Dependency**: Relies on the availability of the central store (Redis/Kvrocks), though load is very low.

## History

The need for distributed unique IDs arose with the explosion of web-scale applications. Traditional database auto-increment keys became a bottleneck in sharded databases. Twitter's **Snowflake** (2010) was a pioneer, using time-based bit manipulation to generate IDs without coordination. However, Snowflake depends heavily on system clocks. Database-based "Segment" approaches (like Flickr's ticket server and later Meituan Leaf's segment mode) emerged to solve clock dependency issues by allocating blocks of IDs. `kvid` follows the Segment pattern, leveraging modern Redis/Kvrocks for speed and atomicity, combined with Rust's safety and concurrency features.