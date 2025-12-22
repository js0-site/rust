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
