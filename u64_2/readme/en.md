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

*   **High 4 bits (Bits 7-4)**: Store the byte length of **Integer A**.
*   **Low 4 bits (Bits 3-0)**: Store the byte length of **Integer B**.

**Length Encoding Rule (-1 offset):**
To maximize space utilization, the stored length value = `actual byte count - 1`.
*   Stored value `0` (0000) $\rightarrow$ represents actual length **1 byte**.
*   Stored value `7` (0111) $\rightarrow$ represents actual length **8 bytes**.

### 2. Data Body (Payload)
Following the Tag is the pure data part, arranged in order:
1.  **Integer A data**: Stored in little-endian order, length determined by Tag high 4 bits.
2.  **Integer B data**: Stored in little-endian order, length determined by Tag low 4 bits.

---

## Encoding/Decoding Logic Explanation

### Encoding Process

1.  **Calculate Length**: Determine the minimum byte count required for integers A and B (range 1-8) by calculating leading zeros.
2.  **Build Tag**:
    *   Subtract 1 from A's length, shift left by 4 bits.
    *   Subtract 1 from B's length.
    *   Perform OR operation on both to generate Tag byte.
3.  **Write Data**:
    *   Write Tag byte.
    *   Write valid bytes of integer A (little-endian) to buffer.
    *   Write valid bytes of integer B (little-endian).

### Decoding Process - Performance Critical

Decoding is the core optimization point of this algorithm, using **Masking** technique to replace traditional byte-by-byte reading.

1.  **Read Tag**: Read the first byte.
2.  **Parse Length**:
    *   Right shift by 4 bits and add 1 to get A's length `LenA`.
    *   Mask low 4 bits and add 1 to get B's length `LenB`.
3.  **Fast Load**:
    *   **Load A**: Using CPU's unaligned read capability, directly load a complete 64-bit word (8 bytes) from the position after Tag, then look up table based on `LenA` and use mask to clear high-bit garbage data.
    *   **Load B**: Calculate B's starting offset (`1 + LenA`), similarly load a complete 64-bit word, look up table based on `LenB` and apply mask.
4.  **Return Result**: Output two restored integers.

---

## Performance Comparison Principle: Why faster than VByte?

### Traditional VByte Pain Points
*   **Branch Dependency**: VByte decoding needs to check the most significant bit (MSB) of each byte to decide whether to continue reading. This means decoding an 8-byte integer might require CPU to make 8 branch predictions.
*   **Pipeline Stall**: Once branch prediction fails, the CPU pipeline will be cleared, causing serious performance penalties.

### Dual-U64 Group Varint Advantages
*   **Deterministic Execution**: After reading Tag, CPU knows exactly what to do next.
*   **Instruction-level Parallelism**: Parsing length and loading data can be optimized by the compiler into parallel instruction sequences.
*   **Reduced Memory Interaction**: By reading 64-bit wide words instead of single bytes, the number of memory bus interactions is reduced.

## Applicable Scenarios
*   Data structures containing fixed **(Key, Value)** pairs both as `u64`.
*   **(DocID, Frequency)** blocks in inverted indexes.
*   Serialization scenarios requiring extremely high throughput with only a small number of integers (2-4).

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