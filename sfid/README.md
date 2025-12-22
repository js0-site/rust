[English](#en) | [中文](#zh)

---

<a id="en"></a>

# sfid : Distributed Snowflake ID Generator with Auto-Allocated Process ID

## Features

- Lock-free atomic ID generation
- Configurable bit layout via `Layout` trait
- Default: 36-bit timestamp (seconds), 11-bit process ID, 17-bit sequence
- Redis-based automatic process ID allocation
- Heartbeat mechanism with auto-release on crash
- Clock drift tolerance (sequence borrowing + warning log)
- Sequence exhaustion handling (timestamp advance)
- Configurable epoch

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

Process ID allocation uses a two-layer mechanism to ensure uniqueness and support fast restarts:

### Local Identity

1. Get machine unique ID via `machine_uid`
2. Try to lock `/tmp/sfid/{app}/{seq}` file (seq = 0, 1, 2, ...)
3. First successful lock determines local sequence number
4. Identity = `{machine_id}:{local_seq}`

This ensures:
- Same machine restarting gets same identity (if same local_seq available)
- Multiple processes on same machine get different identities
- Process crash releases file lock immediately

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
| machine-uid | Machine unique ID |
| fs4 | File locking |
| thiserror | Error handling |
| tracing | Logging |

---

## About

This project is an open-source component of [js0.site ⋅ Refactoring the Internet Plan](https://js0.site).

We are redefining the development paradigm of the Internet in a componentized way. Welcome to follow us:

* [Google Group](https://groups.google.com/g/js0-site)
* [js0site.bsky.social](https://bsky.app/profile/js0site.bsky.social)

---

<a id="zh"></a>

# sfid : 自动分配进程号的分布式雪花 ID 生成器

## 特性

- 无锁原子 ID 生成
- 可配置位布局（`Layout` trait）
- 默认：36 位时间戳（秒）、11 位进程号、17 位序列号
- 基于 Redis 自动分配进程号
- 心跳机制，进程崩溃自动释放
- 时钟回拨容错（序列号借用 + 告警日志）
- 序列号耗尽处理（时间戳推进，借用未来时间）
- 可配置纪元

## 安装

```sh
cargo add sfid
```

指定特性：

```sh
cargo add sfid -F snowflake,auto_pid,parse
```

## 快速开始

### 手动指定进程号

```rust
use sfid::{Snowflake, EPOCH};

let sf = Snowflake::new(EPOCH, 1);
let id = sf.next();
println!("{id}");
```

### 自动分配进程号 (Redis)

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

### 解析 ID

```rust
use sfid::parse;

let parsed = parse(id);
println!("ts: {}, pid: {}, seq: {}", parsed.ts, parsed.pid, parsed.seq);
```

### 自定义位布局

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

## API 参考

### Traits

#### `Layout`

可配置的 ID 位布局。

| 常量 | 说明 |
|------|------|
| `TS_BITS` | 时间戳位数 |
| `PID_BITS` | 进程号位数 |
| `SEQ_BITS` | 序列号位数 |
| `SEQ_MASK` | 派生：`(1 << SEQ_BITS) - 1` |
| `PID_MASK` | 派生：`(1 << PID_BITS) - 1` |
| `TS_MASK` | 派生：`(1 << TS_BITS) - 1` |
| `TS_SHIFT` | 派生：`SEQ_BITS + PID_BITS` |
| `MAX_PID` | 派生：`1 << PID_BITS` |

### 常量

| 名称 | 类型 | 说明 |
|------|------|------|
| `EPOCH` | `u64` | 默认纪元：2025-12-22 00:00:00 UTC（秒） |

### 结构体

#### `Snowflake<L: Layout = DefaultLayout>`

原子状态 ID 生成器。

| 方法 | 说明 |
|------|------|
| `new(epoch, pid)` | 手动指定进程号创建 |
| `auto(app, epoch)` | Redis 自动分配进程号创建 |
| `next()` | 生成下个 ID |

#### `DefaultLayout`

默认位布局：36-11-17。

#### `Pid`

带心跳的进程号句柄，drop 时停止心跳。

| 方法 | 说明 |
|------|------|
| `id()` | 获取分配的进程号 |

#### `ParsedId`

解析后的 ID 组件。

| 字段 | 类型 | 说明 |
|------|------|------|
| `ts` | `u64` | 相对纪元的时间戳偏移（秒） |
| `pid` | `u16` | 进程号 |
| `seq` | `u32` | 序列号 |

### 函数

| 名称 | 说明 |
|------|------|
| `allocate::<L>(app)` | 从 Redis 分配进程号 |
| `parse(id)` | 使用默认布局解析 ID |
| `parse_with::<L>(id)` | 使用自定义布局解析 ID |

## ID 结构（默认布局）

秒精度时间戳的 64 位有符号整数：

```
┌───────┬──────────────────────────┬─────────────┬──────────────┐
│ 1 bit │        36 bits           │   11 bits   │   17 bits    │
│ 符号  │      时间戳（秒）          │   进程号    │    序列号    │
│  (0)  │     (相对纪元偏移)        │  (0-2047)   │  (0-131071)  │
└───────┴──────────────────────────┴─────────────┴──────────────┘
```

- 时间戳：2^36 秒 ≈ **2177 年**（2025-12-22 到 ~4202 年）
- 进程号：2048 并发实例
- 序列号：每实例每秒 131072 ID

## 时钟回拨处理

当时钟回拨时：
- 序列号借用，继续使用上次时间戳
- 回拨超过 1 秒，通过 `tracing::warn` 记录告警
- 序列号耗尽时，时间戳自动推进（借用未来时间）

确保 NTP 校时或虚拟机迁移时 ID 唯一性。

## 进程号分配

进程号分配采用双层机制，确保唯一性并支持快速重启：

### 本地标识

1. 通过 `machine_uid` 获取机器唯一 ID
2. 尝试锁定 `/tmp/sfid/{app}/{seq}` 文件（seq = 0, 1, 2, ...）
3. 首个成功锁定的决定本地序号
4. 标识 = `{machine_id}:{local_seq}`

这确保：
- 同一机器重启后获得相同标识（如果相同 local_seq 可用）
- 同一机器多进程获得不同标识
- 进程崩溃立即释放文件锁

### Redis 注册

使用标识作为 Redis value 进行分布式协调：

```
sfid:{app}:{pid_le_bytes} -> {machine_id}:{local_seq}
```

### 心跳

- 间隔：3 分钟
- 过期：10 分钟
- 进程退出自动释放（Drop trait + 文件锁释放）

## 技术栈

| Crate | 用途 |
|-------|------|
| coarsetime | 快速时间戳获取 |
| fred | Redis 客户端 |
| tokio | 异步运行时 |
| machine-uid | 机器唯一 ID |
| fs4 | 文件锁 |
| thiserror | 错误处理 |
| tracing | 日志 |

---

## 关于

本项目为 [js0.site ⋅ 重构互联网计划](https://js0.site) 的开源组件。

我们正在以组件化的方式重新定义互联网的开发范式，欢迎关注：

* [谷歌邮件列表](https://groups.google.com/g/js0-site)
* [js0site.bsky.social](https://bsky.app/profile/js0site.bsky.social)
