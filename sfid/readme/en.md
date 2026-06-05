# sfid : Distributed Snowflake ID Generator with Auto-Allocated Process ID

## Features

- Lock-free atomic ID generation
- Configurable bit layout via `Layout` trait
- Default: 35-bit timestamp (seconds), 10-bit process ID, 19-bit sequence
- Redis-based automatic process ID allocation
- Heartbeat mechanism with auto-release on crash
- Clock drift tolerance (sequence borrowing + warning log)
- Sequence exhaustion handling (timestamp advance)

## Installation

```sh
cargo add sfid
```

With specific features:

```sh
cargo add sfid -F snowflake,auto_pid,parse
```

## Quick Start

### Manual Process ID

```rust
use sfid::{SfId, EPOCH};

let sf = SfId::new(EPOCH, 1);
let id: u64 = sf.get();
println!("{id}");
```

### Auto-Allocated Process ID (Redis)

```rust
#[tokio::main]
async fn main() -> sfid::Result<()> {
  let sf = sfid::new("myapp").await?;
  let id: u64 = sf.get();
  println!("{id}");
  Ok(())
}
```

### Parse ID

```rust
use sfid::parse;

let id: u64 = 12345678;
let parsed = parse(id);
println!("ts: {}, pid: {}, seq: {}", parsed.ts, parsed.pid, parsed.seq);
```

### Custom Bit Layout

```rust
use sfid::{EPOCH, Layout, SfId, parse_with};

struct MyLayout;
impl Layout for MyLayout {
  const TS_BITS: u32 = 41;
  const PID_BITS: u32 = 10;
  const SEQ_BITS: u32 = 13;
}

let sf = SfId::<MyLayout>::new(EPOCH, 1);
let id: u64 = sf.get();
let parsed = parse_with::<MyLayout>(id);
```

## API Reference

### Traits

#### `Layout`

Configurable bit layout for ID generation.

| Constant   | Description                    |
| ---------- | ------------------------------ |
| `TS_BITS`  | Timestamp bits                 |
| `PID_BITS` | Process ID bits                |
| `SEQ_BITS` | Sequence bits                  |
| `SEQ_MASK` | Derived: `(1 << SEQ_BITS) - 1` |
| `PID_MASK` | Derived: `(1 << PID_BITS) - 1` |
| `TS_MASK`  | Derived: `(1 << TS_BITS) - 1`  |
| `TS_SHIFT` | Derived: `SEQ_BITS + PID_BITS` |
| `MAX_PID`  | Derived: `1 << PID_BITS`       |

### Constants

| Name    | Type  | Description                                      |
| ------- | ----- | ------------------------------------------------ |
| `EPOCH` | `u64` | Default epoch: 2025-12-22 00:00:00 UTC (seconds) |

### Structs

#### `SfId<L: Layout = DefaultLayout>`

ID generator with atomic state.

| Method            | Description                   |
| ----------------- | ----------------------------- |
| `new(epoch, pid)` | Create with manual process ID |
| `get() -> u64`    | Generate ID                   |

#### `DefaultLayout`

Default bit layout: 35-10-19.

#### `Pid`

Process ID handle with heartbeat. Stops heartbeat on drop.

| Method | Description              |
| ------ | ------------------------ |
| `id()` | Get allocated process ID |

#### `ParsedId`

Parsed ID components.

| Field | Type  | Description                           |
| ----- | ----- | ------------------------------------- |
| `ts`  | `u64` | Timestamp offset from epoch (seconds) |
| `pid` | `u16` | Process ID                            |
| `seq` | `u32` | Sequence number                       |

### Functions

| Name                       | Description                                |
| -------------------------- | ------------------------------------------ |
| `allocate::<L>(app)`       | Allocate process ID from Redis             |
| `new(app)`                 | Create SfId with auto-allocated process ID |
| `parse(id: u64)`           | Parse ID with default layout               |
| `parse_with::<L>(id: u64)` | Parse ID with custom layout                |

## ID Structure (Default Layout)

64-bit unsigned integer with second-precision timestamp:

```
┌─────────────────────────┬─────────────┬────────────────┐
│        35 bits          │   10 bits   │    19 bits     │
│    timestamp (sec)      │ process ID  │    sequence    │
│   (offset from epoch)   │  (0-1023)   │  (0-524287)    │
└─────────────────────────┴─────────────┴────────────────┘
```

- Timestamp: 2^35 seconds ≈ **1088 years** (2025-12-22 to ~3113)
- Process ID: 1024 concurrent instances
- Sequence: 524288 IDs per second per instance

## Clock Drift Handling

When clock drifts backward:

- Sequence borrowing continues from last timestamp
- If drift exceeds 1 second, logs warning via `log::warn`
- When sequence exhausted, timestamp advances automatically (borrows future time)

This ensures ID uniqueness even under NTP adjustments or VM migrations.

## Process ID Allocation

Process ID allocation uses a two-layer mechanism to ensure uniqueness and prevent ID exhaustion from rapid restarts.

### Why This Design?

Traditional snowflake implementations generate a new random identifier on each startup. This causes a problem: if a process crashes and restarts repeatedly, it gets a new identifier each time, consuming global process IDs rapidly. With only 2048 slots, frequent restarts could exhaust all available IDs.

Our solution: **persistent machine identity + file locks**. Same machine restarting gets the same identity, so it reclaims its previous Redis slot instead of consuming a new one.

### Local Identity

1. Get or create machine ID via `osid` crate (`hostname:random`, persistently stored)
2. Try to lock `{data_dir}/sfid/{app}/{seq}` file (seq = 0, 1, 2, ...)
3. First successful lock determines local sequence number
4. Identity = `{machine_id}:{local_seq}`

Lock directory is cross-platform persistent (via `osid::dir()`):

- Linux: `~/.local/share/sfid`
- macOS: `~/Library/Application Support/sfid`
- Windows: `C:\Users\<User>\AppData\Local\sfid`

This ensures:

- Same machine restarting gets same identity → reclaims previous Redis slot
- Multiple processes on same machine get different local_seq → different identities
- Process crash releases file lock immediately → slot available for restart

### Redis Registration

Uses identity as Redis value for distributed coordination:

```
sfid:{app}:{pid_le_bytes} -> {machine_id}:{local_seq}
```

### Heartbeat

- Interval: 3 minutes
- Expiration: 10 minutes
- Auto-release on process exit (Drop trait + file lock release)

## Tech Stack

| Crate      | Purpose                       |
| ---------- | ----------------------------- |
| coarsetime | Fast timestamp retrieval      |
| fred       | Redis client                  |
| tokio      | Async runtime                 |
| osid       | Machine ID and data directory |
| fs4        | File locking                  |
| thiserror  | Error handling                |
| log        | Logging                       |
