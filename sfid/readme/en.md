# sfid : Distributed Snowflake ID Generator with Auto-Allocated Process ID

## Features

- Lock-free atomic ID generation
- Configurable bit layout via `Layout` trait
- Default: 36-bit timestamp (seconds), 11-bit process ID, 17-bit sequence
- Redis-based automatic process ID allocation
- Heartbeat mechanism with auto-release on crash
- Clock drift tolerance (sequence borrowing + warning log)
- Sequence exhaustion handling (timestamp advance)
- Cross-platform persistent lock directory

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
use sfid::{Snowflake, EPOCH};

let sf = Snowflake::new(EPOCH, 1);
let id = sf.next();
println!("{id}");
```

### Auto-Allocated Process ID (Redis)

```rust
use sfid::{Snowflake, EPOCH};

#[tokio::main]
async fn main() -> sfid::Result<()> {
  let sf = Snowflake::auto("myapp", EPOCH).await?;
  let id = sf.next();
  println!("{id}");
  Ok(())
}
```

### Parse ID

```rust
use sfid::parse;

let parsed = parse(id);
println!("ts: {}, pid: {}, seq: {}", parsed.ts, parsed.pid, parsed.seq);
```

### Custom Bit Layout

```rust
use sfid::{Layout, Snowflake, parse_with};

struct MyLayout;
impl Layout for MyLayout {
  const TS_BITS: u32 = 41;
  const PID_BITS: u32 = 10;
  const SEQ_BITS: u32 = 13;
}

let sf = Snowflake::<MyLayout>::new(my_epoch, 1);
let id = sf.next();
let parsed = parse_with::<MyLayout>(id);
```

## API Reference

### Traits

#### `Layout`

Configurable bit layout for ID generation.

| Constant | Description |
|----------|-------------|
| `TS_BITS` | Timestamp bits |
| `PID_BITS` | Process ID bits |
| `SEQ_BITS` | Sequence bits |
| `SEQ_MASK` | Derived: `(1 << SEQ_BITS) - 1` |
| `PID_MASK` | Derived: `(1 << PID_BITS) - 1` |
| `TS_MASK` | Derived: `(1 << TS_BITS) - 1` |
| `TS_SHIFT` | Derived: `SEQ_BITS + PID_BITS` |
| `MAX_PID` | Derived: `1 << PID_BITS` |

### Constants

| Name | Type | Description |
|------|------|-------------|
| `EPOCH` | `u64` | Default epoch: 2025-12-22 00:00:00 UTC (seconds) |

### Structs

#### `Snowflake<L: Layout = DefaultLayout>`

ID generator with atomic state.

| Method | Description |
|--------|-------------|
| `new(epoch, pid)` | Create with manual process ID |
| `auto(app, epoch)` | Create with Redis-allocated process ID |
| `next()` | Generate next ID |

#### `DefaultLayout`

Default bit layout: 36-11-17.

#### `Pid`

Process ID handle with heartbeat. Stops heartbeat on drop.

| Method | Description |
|--------|-------------|
| `id()` | Get allocated process ID |

#### `ParsedId`

Parsed ID components.

| Field | Type | Description |
|-------|------|-------------|
| `ts` | `u64` | Timestamp offset from epoch (seconds) |
| `pid` | `u16` | Process ID |
| `seq` | `u32` | Sequence number |

### Functions

| Name | Description |
|------|-------------|
| `allocate::<L>(app)` | Allocate process ID from Redis |
| `parse(id)` | Parse ID with default layout |
| `parse_with::<L>(id)` | Parse ID with custom layout |

## ID Structure (Default Layout)

64-bit signed integer with second-precision timestamp:

```
┌───────┬──────────────────────────┬─────────────┬──────────────┐
│ 1 bit │        36 bits           │   11 bits   │   17 bits    │
│ sign  │    timestamp (sec)       │ process ID  │   sequence   │
│  (0)  │   (offset from epoch)    │  (0-2047)   │  (0-131071)  │
└───────┴──────────────────────────┴─────────────┴──────────────┘
```

- Timestamp: 2^36 seconds ≈ **2177 years** (2025-12-22 to ~4202)
- Process ID: 2048 concurrent instances
- Sequence: 131072 IDs per second per instance

## Clock Drift Handling

When clock drifts backward:
- Sequence borrowing continues from last timestamp
- If drift exceeds 1 second, logs warning via `tracing::warn`
- When sequence exhausted, timestamp advances automatically (borrows future time)

This ensures ID uniqueness even under NTP adjustments or VM migrations.

## Process ID Allocation

Process ID allocation uses a two-layer mechanism to ensure uniqueness and prevent ID exhaustion from rapid restarts.

### Why This Design?

Traditional snowflake implementations generate a new random identifier on each startup. This causes a problem: if a process crashes and restarts repeatedly, it gets a new identifier each time, consuming global process IDs rapidly. With only 2048 slots, frequent restarts could exhaust all available IDs.

Our solution: **persistent machine identity + file locks**. Same machine restarting gets the same identity, so it reclaims its previous Redis slot instead of consuming a new one.

### Local Identity

1. Get or create machine ID (`hostname:random`, stored persistently)
2. Try to lock `{data_dir}/sfid/{app}/{seq}` file (seq = 0, 1, 2, ...)
3. First successful lock determines local sequence number
4. Identity = `{machine_id}:{local_seq}`

Lock directory is cross-platform persistent:
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

| Crate | Purpose |
|-------|---------|
| coarsetime | Fast timestamp retrieval |
| fred | Redis client |
| tokio | Async runtime |
| hostname | Get hostname |
| fs4 | File locking |
| dirs | Cross-platform directories |
| thiserror | Error handling |
| tracing | Logging |
