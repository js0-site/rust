[English](#en) | [中文](#zh)

---

<a id="en"></a>

![u64_2 English Logo](https://raw.githubusercontent.com/js0-site/rust/refs/heads/main/u64_2/svg/en.svg)

# u64_2

u64_2 is a highly customized variable-length encoding scheme specifically designed for **simultaneously storing two `u64` integers**.

Compared to standard VByte (Varint/LEB128), this scheme eliminates branch loops in the decoding process by grouping metadata (length information), thereby maximizing the utilization of modern CPU pipelines and branch prediction capabilities (such as Apple Silicon M series). It maintains high compression ratios while providing decoding speeds close to memory copy (Memcpy) levels.

## Core Advantages

1.  **Ultra-fast Decoding**: Completely branchless logic without `while` loops, decoding requires only simple bitwise operations and memory reads.
2.  **Compact Storage**: Minimum 3 bytes (when both integers are less than 256), maximum 17 bytes.
3.  **Hardware-friendly**: Optimized for CPUs supporting unaligned memory access (such as ARM64/x64), can load data in one go using registers and mask clean.

## Data Layout (Bit Layout)

The encoded data stream consists of a **Tag (tag byte)** and subsequent **Data Body (data body)**.

Structure diagram:
`[Tag (1 Byte)] [Integer A Bytes...] [Integer B Bytes...]`

### 1. Tag Byte (Header)

Tag is the first byte of the data stream, which records the byte length information of both integers simultaneously. Tag is split into high 4 bits and low 4 bits:

- **High 4 bits (Bits 7-4)**: Store the byte length of **Integer A**.
- **Low 4 bits (Bits 3-0)**: Store the byte length of **Integer B**.

**Length Encoding Rule (-1 offset):**
To maximize space utilization, the stored length value = `actual byte count - 1`.

- Stored value `0` (0000) $\rightarrow$ represents actual length **1 byte**.
- Stored value `7` (0111) $\rightarrow$ represents actual length **8 bytes**.

### 2. Data Body (Payload)

Following the Tag is the pure data part, arranged in order:

1.  **Integer A data**: Stored in little-endian order, length determined by Tag high 4 bits.
2.  **Integer B data**: Stored in little-endian order, length determined by Tag low 4 bits.

---

## Encoding/Decoding Logic Explanation

### Encoding Process

1.  **Calculate Length**: Determine the minimum byte count required for integers A and B (range 1-8) by calculating leading zeros.
2.  **Build Tag**:
    - Subtract 1 from A's length, shift left by 4 bits.
    - Subtract 1 from B's length.
    - Perform OR operation on both to generate Tag byte.
3.  **Write Data**:
    - Write Tag byte.
    - Write valid bytes of integer A (little-endian) to buffer.
    - Write valid bytes of integer B (little-endian).

### Decoding Process - Performance Critical

Decoding is the core optimization point of this algorithm, using **Masking** technique to replace traditional byte-by-byte reading.

1.  **Read Tag**: Read the first byte.
2.  **Parse Length**:
    - Right shift by 4 bits and add 1 to get A's length `LenA`.
    - Mask low 4 bits and add 1 to get B's length `LenB`.
3.  **Fast Load**:
    - **Load A**: Using CPU's unaligned read capability, directly load a complete 64-bit word (8 bytes) from the position after Tag, then look up table based on `LenA` and use mask to clear high-bit garbage data.
    - **Load B**: Calculate B's starting offset (`1 + LenA`), similarly load a complete 64-bit word, look up table based on `LenB` and apply mask.
4.  **Return Result**: Output two restored integers.

---

## Performance Comparison Principle: Why faster than VByte?

### Traditional VByte Pain Points

- **Branch Dependency**: VByte decoding needs to check the most significant bit (MSB) of each byte to decide whether to continue reading. This means decoding an 8-byte integer might require CPU to make 8 branch predictions.
- **Pipeline Stall**: Once branch prediction fails, the CPU pipeline will be cleared, causing serious performance penalties.

### Dual-U64 Group Varint Advantages

- **Deterministic Execution**: After reading Tag, CPU knows exactly what to do next.
- **Instruction-level Parallelism**: Parsing length and loading data can be optimized by the compiler into parallel instruction sequences.
- **Reduced Memory Interaction**: By reading 64-bit wide words instead of single bytes, the number of memory bus interactions is reduced.

## Applicable Scenarios

- Data structures containing fixed **(Key, Value)** pairs both as `u64`.
- **(DocID, Frequency)** blocks in inverted indexes.
- Serialization scenarios requiring extremely high throughput with only a small number of integers (2-4).

---

## Usage

### Encoding

```rust
use u64_2::encode;

let mut buffer = [0u8; 32];
let num1: u64 = 500;        // Needs 2 bytes
let num2: u64 = 100000;     // Needs 3 bytes

let len = encode(num1, num2, &mut buffer);
// Encoded data is in &buffer[..len]
```

### Decoding

```rust
use u64_2::decode;

let encoded_data = [0x12, 0xF4, 0x01, 0xA0, 0x86, 0x01];
let (num1, num2, consumed) = decode(&encoded_data);
// num1 = 500, num2 = 100000, consumed = 6
```

---

## Performance Benchmarks

For detailed performance benchmark results, please refer to [benches/RESULTS.md](../benches/RESULTS.md).

Run benchmarks:

```bash
cargo bench --bench u64_encode_decode
```

Key performance indicators:

- Encoding time: ~4.5 ns
- Decoding time: ~5.3 ns
- Throughput: ~180M elements/s

---

## License

This project is licensed under [MulanPSL-2.0](LICENSE).

## Bench

## u64_2 vs vb Benchmark

Comparing u64_2 (pair encoding) with vb (varint) using 100,000 integers (mixed distribution: 60% small, 30% medium, 10% large).

### Results

| Library | Encode (M/s) | Decode (M/s) |
| ------- | ------------ | ------------ |
| u64_2   | 2727.6       | 2258.2       |
| vb      | 199.5        | 288.3        |

### Environment

macOS 26.1 (arm64) · Apple M2 Max · 12 cores · 64.0GB · rustc 1.94.0-nightly (21ff67df1 2025-12-15)

---

## About

This project is an open-source component of [js0.site ⋅ Refactoring the Internet Plan](https://js0.site).

We are redefining the development paradigm of the Internet in a componentized way. Welcome to follow us:

- [Google Group](https://groups.google.com/g/js0-site)
- [js0site.bsky.social](https://bsky.app/profile/js0site.bsky.social)

---

<a id="zh"></a>

![u64_2 中文标志](https://raw.githubusercontent.com/js0-site/rust/refs/heads/main/u64_2/svg/zh.svg)

# u64_2

u64_2 是一种高度定制化的变长编码方案，专为**同时存储两个 `u64` 整数**而设计。

与标准的 VByte (Varint/LEB128) 相比，本方案通过将元数据（长度信息）分组存储，彻底消除了解码过程中的分支循环（Branch Loops），从而最大限度地利用现代 CPU（如 Apple Silicon M 系列）的流水线和分支预测能力。它在保持高压缩率的同时，提供接近内存拷贝（Memcpy）级别的解码速度。

## 核心优势

1.  **极速解码**：完全无分支（Branchless）逻辑，无 `while` 循环，解码仅需简单的位运算和内存读取。
2.  **紧凑存储**：最小仅占用 3 字节（两个整数均小于 256 时），最大占用 17 字节。
3.  **硬件友好**：针对支持非对齐内存访问（Unaligned Access）的 CPU（如 ARM64/x64）进行了优化，可利用寄存器一次性加载数据并掩码清洗。

## 数据布局 (Bit Layout)

编码后的数据流由一个 **Tag（标签字节）** 和随后的 **Data Body（数据体）** 组成。

结构示意图：
`[Tag (1 Byte)] [Integer A Bytes...] [Integer B Bytes...]`

### 1. Tag 字节 (Header)

Tag 是数据流的第一个字节，它同时记录了两个整数的字节长度信息。Tag 被拆分为高 4 位和低 4 位：

- **高 4 位 (Bits 7-4)**：存储 **整数 A** 的字节长度。
- **低 4 位 (Bits 3-0)**：存储 **整数 B** 的字节长度。

**长度编码规则（-1 偏移）：**
为了最大化利用空间，存储的长度值 = `实际字节数 - 1`。

- 存储值 `0` (0000) $\rightarrow$ 代表实际长度 **1 字节**。
- 存储值 `7` (0111) $\rightarrow$ 代表实际长度 **8 字节**。

### 2. Data Body (Payload)

紧随 Tag 之后的是纯数据部分，按顺序排列：

1.  **整数 A 的数据**：按小端序（Little Endian）存储，长度由 Tag 高 4 位决定。
2.  **整数 B 的数据**：按小端序（Little Endian）存储，长度由 Tag 低 4 位决定。

---

## 编解码逻辑说明

### 编码流程 (Encoding)

1.  **计算长度**：通过计算整数的前导零（Leading Zeros），确定整数 A 和整数 B 各自需要的最小字节数（范围 1-8）。
2.  **构建 Tag**：
    - 将 A 的长度减 1，左移 4 位。
    - 将 B 的长度减 1。
    - 将两者进行"或"运算（OR），生成 Tag 字节。
3.  **写入数据**：
    - 写入 Tag 字节。
    - 将整数 A 的有效字节（小端序）写入缓冲区。
    - 紧接着写入整数 B 的有效字节（小端序）。

### 解码流程 (Decoding) - 性能关键

解码是本算法的核心优化点，采用 **Masking（掩码）** 技术替代传统的字节逐个读取。

1.  **读取 Tag**：读取第一个字节。
2.  **解析长度**：
    - 右移 4 位并加 1，得到 A 的长度 `LenA`。
    - 低 4 位掩码并加 1，得到 B 的长度 `LenB`。
3.  **快速加载 (Fast Load)**：
    - **加载 A**：利用 CPU 的非对齐读取能力，直接从 Tag 之后的位置加载一个完整的 64 位字（8 字节），然后根据 `LenA` 查表，使用掩码将高位垃圾数据清零。
    - **加载 B**：计算出 B 的起始偏移量（`1 + LenA`），同样加载一个完整的 64 位字，根据 `LenB` 查表并应用掩码。
4.  **返回结果**：输出两个还原的整数。

---

## 性能对比原理：为什么比 VByte 快？

### 传统 VByte 的痛点

- **分支依赖**：VByte 解码每一个字节都需要检查最高位（MSB）来决定是否继续读取。这意味着解码一个 8 字节整数可能需要 CPU 做 8 次分支预测。
- **流水线停顿**：一旦分支预测失败，CPU 流水线会被清空，导致严重的性能惩罚。

### Dual-U64 Group Varint 的优势

- **确定性执行**：读取 Tag 后，CPU 精确知道接下来该做什么。
- **指令级并行**：解析长度和加载数据可以被编译器优化为并行的指令序列。
- **减少内存交互**：通过读取 64 位宽字（Word）而非单个字节，减少了内存总线的交互次数。

## 适用场景

- 数据结构中含有固定的 **(Key, Value)** 均为 `u64` 的 Pair 对。
- 倒排索引中的 **(DocID, Frequency)** 块。
- 需要极高吞吐量且只有少量整数（2-4个）的序列化场景。

---

## 使用方法

### 编码

```rust
use u64_2::encode;

let mut buffer = [0u8; 32];
let num1: u64 = 500;        // 需要 2 字节
let num2: u64 = 100000;     // 需要 3 字节

let len = encode(num1, num2, &mut buffer);
// 编码后的数据在 &buffer[..len] 中
```

### 解码

```rust
use u64_2::decode;

let encoded_data = [0x12, 0xF4, 0x01, 0xA0, 0x86, 0x01];
let (num1, num2, consumed) = decode(&encoded_data);
// num1 = 500, num2 = 100000, consumed = 6
```

---

## 性能评测

详细的性能评测结果请参考 [benches/RESULTS.md](../benches/RESULTS.md)。

运行评测：

```bash
cargo bench --bench u64_encode_decode
```

关键性能指标：

- 编码时间：~4.5 ns
- 解码时间：~5.3 ns
- 吞吐量：~180M elements/s

---

## 许可证

本项目采用 [MulanPSL-2.0](LICENSE) 许可证。

## 评测

## u64_2 vs vb 性能评测

对比 u64_2（双值编码）与 vb（变长编码），测试数据：100,000 个整数（混合分布：60% 小值，30% 中值，10% 大值）。

### 结果

| 库    | 编码 (百万/秒) | 解码 (百万/秒) |
| ----- | -------------- | -------------- |
| u64_2 | 2727.6         | 2258.2         |
| vb    | 199.5          | 288.3          |

### 环境

macOS 26.1 (arm64) · Apple M2 Max · 12 核 · 64.0GB · rustc 1.94.0-nightly (21ff67df1 2025-12-15)

---

## 关于

本项目为 [js0.site ⋅ 重构互联网计划](https://js0.site) 的开源组件。

我们正在以组件化的方式重新定义互联网的开发范式，欢迎关注：

- [谷歌邮件列表](https://groups.google.com/g/js0-site)
- [js0site.bsky.social](https://bsky.app/profile/js0site.bsky.social)
