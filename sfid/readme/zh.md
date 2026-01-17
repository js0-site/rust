# sfid : 自动分配进程号的分布式雪花 ID 生成器

## 特性

- 无锁原子 ID 生成
- 可配置位布局（`Layout` trait）
- 默认：35 位时间戳（秒）、10 位进程号、19 位序列号
- 基于 Redis 自动分配进程号
- 心跳机制，进程崩溃自动释放
- 时钟回拨容错（序列号借用 + 告警日志）
- 序列号耗尽处理（时间戳推进，借用未来时间）

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
use sfid::{SfId, EPOCH};

let sf = SfId::new(EPOCH, 1);
let id: u64 = sf.get();
println!("{id}");
```

### 自动分配进程号 (Redis)

```rust
#[tokio::main]
async fn main() -> sfid::Result<()> {
  let sf = sfid::new("myapp").await?;
  let id: u64 = sf.get();
  println!("{id}");
  Ok(())
}
```

### 解析 ID

```rust
use sfid::parse;

let id: u64 = 12345678;
let parsed = parse(id);
println!("ts: {}, pid: {}, seq: {}", parsed.ts, parsed.pid, parsed.seq);
```

### 自定义位布局

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

#### `SfId<L: Layout = DefaultLayout>`

原子状态 ID 生成器。

| 方法 | 说明 |
|------|------|
| `new(epoch, pid)` | 手动指定进程号创建 |
| `get() -> u64` | 生成 ID |

#### `DefaultLayout`

默认位布局：35-10-19。

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
| `new(app)` | 创建自动分配进程号的 SfId |
| `parse(id: u64)` | 使用默认布局解析 ID |
| `parse_with::<L>(id: u64)` | 使用自定义布局解析 ID |

## ID 结构（默认布局）

秒精度时间戳的 64 位无符号整数：

```
┌─────────────────────────┬─────────────┬────────────────┐
│        35 bits          │   10 bits   │    19 bits     │
│      时间戳（秒）         │   进程号    │    序列号      │
│     (相对纪元偏移)       │  (0-1023)   │  (0-524287)    │
└─────────────────────────┴─────────────┴────────────────┘
```

- 时间戳：2^35 秒 ≈ **1088 年**（2025-12-22 到 ~3113 年）
- 进程号：1024 并发实例
- 序列号：每实例每秒 524288 ID

## 时钟回拨处理

当时钟回拨时：
- 序列号借用，继续使用上次时间戳
- 回拨超过 1 秒，通过 `log::warn` 记录告警
- 序列号耗尽时，时间戳自动推进（借用未来时间）

确保 NTP 校时或虚拟机迁移时 ID 唯一性。

## 进程号分配

进程号分配采用双层机制，确保唯一性并防止快速重启导致 ID 耗尽。

### 为何这样设计？

传统雪花实现每次启动都生成新的随机标识。这会导致问题：如果进程反复崩溃重启，每次都获得新标识，快速消耗全局进程号。只有 2048 个槽位，频繁重启可能耗尽所有可用 ID。

我们的方案：**持久化机器标识 + 文件锁**。同一机器重启后获得相同标识，因此会回收之前的 Redis 槽位，而不是消耗新的。

### 本地标识

1. 通过 `osid` 获取或创建机器 ID（`主机名:随机数`，持久化存储）
2. 尝试锁定 `{data_dir}/sfid/{app}/{seq}` 文件（seq = 0, 1, 2, ...）
3. 首个成功锁定的决定本地序号
4. 标识 = `{machine_id}:{local_seq}`

锁目录跨平台持久化（通过 `osid::dir()`）：
- Linux: `~/.local/share/sfid`
- macOS: `~/Library/Application Support/sfid`
- Windows: `C:\Users\<User>\AppData\Local\sfid`

这确保：
- 同一机器重启后获得相同标识 → 回收之前的 Redis 槽位
- 同一机器多进程获得不同 local_seq → 不同标识
- 进程崩溃立即释放文件锁 → 槽位可供重启使用

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
| osid | 机器 ID 和数据目录 |
| fs4 | 文件锁 |
| thiserror | 错误处理 |
| log | 日志 |
