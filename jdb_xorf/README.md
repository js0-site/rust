[English](n) | [中文](#zh)

---

<a id="en"></a>


./readme/en.md

## Bench

## Performance Benchmark

| Library | Filter | Bf Ops | Query Ops | Memory | Speedup |
| --- | --- | --- | --- | --- | --- |
| xorf | Bf8 | 3985.84 | 52562.99 | 116.04 KB | - |
| xorf | BinaryFuse16 | 4036.86 | 48915.75 | 232.04 KB | - |
| xorf | BinaryFuse32 | 3931.59 | 48225.88 | 464.04 KB | - |
| jdb | Bf16 | 4599.84 | 76251.81 | 232.04 KB | - |
| jdb | Bf32 | 4767.73 | 66797.11 | 464.04 KB | - |
| jdb | Bf8 | 4742.57 | 86450.03 | 116.04 KB | - |

## Accuracy

| Library | Filter | False Positive Rate | False Negative Rate |
| --- | --- | --- | --- |
| xorf | Bf8 | 0.39252% | 0 |
| xorf | BinaryFuse16 | 0.00157% | 0 |
| xorf | BinaryFuse32 | 0.00000% | 0 |
| jdb | Bf16 | 0.00144% | 0 |
| jdb | Bf32 | 0.00000% | 0 |
| jdb | Bf8 | 0.39059% | 0 |

---

## About

This project is an open-source component of [js0.site ⋅ Refactoring the Internet Plan](https://js0.site).

We are redefining the development paradigm of the Internet in a componentized way. Welcome to follow us:

* [Google Group](https://groups.google.com/g/js0-site)
* [js0site.bsky.social](https://bsky.app/profile/js0site.bsky.social)

---

<a id="zh"></a>

# jdb_xorf : 极致性能的 Rust Binary Fuse 过滤器

## 项目介绍
jdb_xorf 是针对 Rust 开发的高性能 Binary Fuse 过滤器实现。此类概率型数据结构相较于 Bloom 或 Cuckoo 过滤器，具备更快的查询速度与更小的内存占用。Binary Fuse 过滤器代表了目前静态集合成员检测技术的最高水平。

Binary Fuse 是 Xor Filter 系列的巅峰之作，也是目前已知**最高效**的静态成员检测结构。相比于其他过滤器，它在所有关键指标上都具有显著优势：

### 为什么选择 Binary Fuse？
- **构建速度更快**: 采用了**图分区 (Graph Partitioning)** 技术，将问题拆解为适应 L1/L2 缓存的小型块，构建速度比传统 Xor Filter 快 **10-20 倍**。
- **更节省空间**: 相同误报率下空间占用更低。`Bf8` 仅需约 8.64 bits/entry 即可达到约 0.39% 的误报率（空间开销仅为理论下限的 **1.08x**）。
- **更好的局部性**: 分区设计极大幅度减少了 CPU 缓存失效。
- **极速查询**: 查询是严格的 **O(1)**，仅需 3 次内存访问 + 1 次哈希混淆 + 2 次 XOR 计算。
- **无误漏**: 保证如果元素在集合中，一定会返回 True。

![Performance Benchmark](https://raw.githubusercontent.com/js0-site/rust/heads/main/jdb_xorf/readme/zh.bench.svg)

## 目录
- [注意事项与前提条件](#注意事项与前提条件)
- [使用演示](#使用演示)
- [特性介绍](#特性介绍)
- [设计思路](#设计思路)
- [技术堆栈](#技术堆栈)
- [目录结构](#目录结构)
- [API 说明](#api-说明)
- [构建失败概率](#构建失败概率)
- [历史背景](#历史背景)
- [参考引用](#参考引用)


## 注意事项与前提条件

### 严禁重复元素
Binary Fuse 过滤器 (`Bf8`, `Bf16`, `Bf32`) 的构建算法有一个严格的前提条件：输入的数据结构中**不得包含重复的键 (duplicate keys)**。如果输入的 `u64` 哈希值存在重复，构建过程几乎肯定会失败。如果您直接使用原始过滤器，必须在构建前自行去除重复项。

### Bf 自动去重
推荐使用 `Bf` 包装器来处理任意类型（如 `String`, `&[u8]` 等）。`Bf` 会在内部**自动处理所有的哈希计算、排序和去重**工作，确保构建成功率。您只需传入数据，剩下的交给它处理。

## 使用演示

### 基础 Binary Fuse 过滤器
```rust
use jdb_xorf::{Filter, Bf8};

let keys = vec![1u64, 2, 3];
let filter = Bf8::from(&keys);

assert!(filter.has(&1));
assert!(!filter.has(&4));
```

### 任意类型的构建 (如字符串)
```rust
use jdb_xorf::{Filter, Bf, Bf8};

let fruits = vec!["apple".to_string(), "banana".to_string()];
// Bf 会自动处理哈希和去重。
// 默认使用 RapidHasher 以获得极高性能。
let filter: Bf<String, Bf8> = Bf::from(&fruits);

assert!(filter.has("apple"));
```

### 二进制串 / 字节流构建
```rust
use jdb_xorf::{Filter, Bf, Bf8};

let data: Vec<&[u8]> = vec![b"raw_bytes_1", b"raw_bytes_2"];
let filter: Bf<&[u8], Bf8> = Bf::from(&data);

assert!(filter.has(&b"raw_bytes_1"[..]));
```

### 序列化与反序列化 (可选特性)
开启 `bitcode` 特性后，可直接使用 `bitcode::encode` / `bitcode::decode` 进行序列化。

```rust
use jdb_xorf::{Bf, Bf8};

// 1. 序列化 (Encode)
let keys = vec!["apple", "banana"];
// 无需转换为 String，直接使用 &str
let filter: Bf<&str, Bf8> = Bf::from(&keys);

// 直接使用 bitcode 库函数
let bytes = bitcode::encode(&filter);

// 2. 反序列化 (Decode)
// 能够完全还原类型信息，包括泛型参数
let loaded: Bf<&str, Bf8> = bitcode::decode(&bytes).expect("Decode failed");

assert!(loaded.has("apple"));
```

## 特性介绍
- **极速**: 皮秒级查询延迟。
- **高效**: 空间利用率极高（Bf8 每条目仅需约 8.64 bit，空间开销为理论下限的 1.08x）。
- **灵活**: 提供 `Bf` 适配器，支持非 u64 类型并**自动去重**。
- **便携**: 完整支持 `no_std`，适用于嵌入式环境。
- **序列化**: 可选支持 `bitcode` 或 DMA 零拷贝加载。

## 算法细节 (Mermaid)

### 1. 构造阶段 (Peeling Phase)
```mermaid
graph TD
    Start["开始构造"] --> Init["计算参数: seg_len, capacity"]
    Init --> SeedIter["尝试下一种子 (Seed)"]
    SeedIter --> Mapping["映射键: 计算 3 个槽位 h0, h1, h2"]
    Mapping --> Bucketing["更新桶状态: t2count++ / t2hash XOR= hash"]
    Bucketing --> FindAlone["扫描桶: 寻找 count == 1 的孤立桶"]
    FindAlone --> Queue["加入 alone 队列"]
    Queue --> PeelLoop{"队列是否为空?"}
    PeelLoop -- "否" --> Pop["弹出桶索引, 将键压入 reverse_order 栈"]
    Pop --> Update["更新相邻 2 个桶: 减少计数并异或哈希和"]
    Update --> NewAlone{"产生新孤立桶?"}
    NewAlone -- "是" --> Queue
    NewAlone -- "否" --> PeelLoop
    PeelLoop -- "是" --> Success{"是否处理了所有键?"}
    Success -- "否" --> SeedIter
    Success -- "是" --> Done["进入求解阶段"]
```

### 2. 求解阶段 (Solver Phase)
```mermaid
graph TD
    SStart["开始求解"] --> SInit["初始化指纹数组 fingerprints"]
    SInit --> PopStack["从 reverse_order 栈顶弹出键与槽位信息"]
    PopStack --> ReadOther["读取另外 2 个已确定或初始的指纹"]
    ReadOther --> Assign["计算当前指纹: fp = target_f XOR fp_other1 XOR fp_other2"]
    Assign --> Next{"栈是否为空?"}
    Next -- "否" --> PopStack
    Next -- "是" --> SDone["BinaryFuse 构建成功"]
```

### 3. 查询阶段 (Query Phase)
```mermaid
graph TD
    QKey["输入查询键"] --> QHash["mix64 哈希混淆"]
    QHash --> QSlots["确定 3 个槽位: h0, h1, h2"]
    QSlots --> QRead["原子读取: fp0, fp1, fp2"]
    QRead --> QXor["异或运算: res = fp0 XOR fp1 XOR fp2"]
    QXor --> QMatch{"res == (hash as Fingerprint)?"}
    QMatch -- "是" --> QPres["可能存在 (Probably)"]
    QMatch -- "否" --> QNot["绝对不存在 (Definitely)"]
```

1. **哈希化**: 使用 RapidHash 或自定义哈希器对键进行混淆。
2. **映射**: 根据哈希值在分区图中确定三个槽位。
3. **查找**: 通过对这三个槽位的指纹进行 XOR 运算，判断成员身份。

## 技术堆栈
- **语言**: Rust (Edition 2024)。
- **核心算法**: 二进制分区保险丝图 (Binary-partitioned Fuse Graph) 算法。
- **哈希算法**: RapidHash (基于 `rapidhash` crate), 高质量混淆函数 `mix64`。
- **性能评估**: Criterion 微基准测试。

## 目录结构
- `src/`: 核心实现。
  - `base.rs`: 泛型 Binary Fuse 算法实现。
  - `bfuse*.rs`: 特定指纹宽度的 Binary Fuse 变体 (8, 16, 32-bit)。
  - `bf/`: 构建工具 (包含 `Bf` 适配器)。
  - `hash.rs`: 哈希器与高质量混淆函数实现。
- `benches/`: 性能基准测试集。
- `analysis/`: 均匀性与零分布分析工具。

## API 说明

### Trait
- `Filter<T>`: 成员检测核心 trait。
  - `has(&self, key: &T) -> bool`
  - `len(&self) -> usize`
- `FilterRef<'a, T>`: 过滤器数据的零拷贝引用。
- `DmaSerializable`: 适用于直接内存访问 (DMA) 的序列化接口。

### 类型
- `Bf8`, `Bf16`, `Bf32`: 托管内存的过滤器。
- `Bf8Ref`, `Bf16Ref`, `Bf32Ref`: 借用内存的过滤器。
- `Bf<T, F, H = RapidHasher>`: 通用包装器，使用哈希器 `H` 与过滤器 `F` 处理任意类型 `T`，具备自动去重功能。

### 总结对比表

| 过滤器 | 内存占用 | 查询速度 | 构建速度 | 缓存友好度 | 场景 |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Binary Fuse** | **极低 (≈1.08x 理论下限)** | **极快 (3 次访问)** | **极快 (分区优化)** | **优秀** | 静态海量数据最佳选择 |
| **Xor Filter** | 低 | 快 | 慢 | 差 | 旧一代方案 |
| **Bloom Filter** | 中 | 慢 (多次哈希) | 快 | 差 | 动态数据/简单场景 |
| **Cuckoo Filter** | 低 | 中 (随机探测) | 慢 | 差 | 需要支持删除 |

## 构建失败概率

Binary Fuse 过滤器构建失败的理论概率极低。本库在构建时会自动进行 1000 次重试（每次使用不同的随机种子）。

根据 Mueller & Lemire 的论文 [[2]](#参考引用)，单次构建的成功率下界为 **90%**。而基于本库的实测数据（针对 100,000 个随机键进行 1000 轮测试）：
- **一次性构建成功率**: **98.70%** (1000 次中 987 次首次成功)
- **平均尝试次数**: **1.013**

这意味着单次构建的失败率仅为 ![](https://r.latexeasy.com/image.svg?P_{fail}%20\approx%201.3\%25%20=%200.013)。

因此，连续 1000 次构建均失败的概率为：
![](https://r.latexeasy.com/image.svg?P_{total\_fail}%20=%20(P_{fail})^{1000}%20\approx%20(0.013)^{1000}%20\approx%2010^{-1880})

**![](https://r.latexeasy.com/image.svg?10^{-1880}) 是一个完全可以视为 0 的数值。**
它比宇宙中的原子总数倒数还要小得多，更比现代计算机硬件发生不可纠正错误的概率（约为 ![](https://r.latexeasy.com/image.svg?10^{-11})) 低几百个数量级。

Google 在大规模数据中心的研究表明，每年约有 **1.3%** 的机器经历过至少一次不可纠正的内存错误 (Uncorrectable Error)。假设一次构建过程耗时 0.1 秒，那么在此期间发生硬件错误的概率约为 ![](https://r.latexeasy.com/image.svg?10^{-11}) 量级。

| 事件类型 | 近似概率 | 风险定性 |
| :--- | :--- | :--- |
| **构建期间硬件位翻转** | ![](https://r.latexeasy.com/image.svg?\approx%2010^{-11}) | 真实存在的极低风险 |
| **Binary Fuse 构建失败** | ![](https://r.latexeasy.com/image.svg?\approx%2010^{-1880}) | 物理上的“绝对不可能” |

因此，库的设计原则是：**将构建失败视为不可恢复的致命错误（panic），而非运行时错误（Result/TryFrom）。**

如果您在构建过程中遇到了 panic，更有可能是因为：
1. **输入数据存在重复键**（这是最常见的原因，即使您认为已经去重）。
2. **硬件故障**（内存位翻转等）。
3. **极度罕见的概率事件**（此时只需简单的重试即可，但在人类文明的时间尺度内几乎不可能遇到）。

出于易用性考虑，我们优先使用 `From` trait，因为它符合绝大多数场景下的心理预期：构建过程总是成功的。

## 历史背景
概率过滤器的技术演进从 Bloom 过滤器 (1970) 开始，历经 Cuckoo 过滤器 (2014) 的改进。2020 年 Xor 过滤器的出现带来了范式转移，通过完美的 XOR 求和实现更优性能。2022 年，Mueller 与 Lemire 进一步提出 Binary Fuse 过滤器，通过图分区技术使其在空间和时间效率上逼近了理论极限。

## 参考引用

- [Xor Filters: Faster and Smaller Than Bloom and Cuckoo Filters](https://arxiv.org/abs/1912.08258)
- [Binary Fuse Filters: Fast and Smaller Than Xor Filters](https://arxiv.org/abs/2201.01171)
- [Fuse Graph](https://arxiv.org/abs/1907.04749)
- [Go 实现](https://github.com/FastFilter/xorfilter)
- [C 实现](https://github.com/FastFilter/xor_singleheader)
- [fuse graph](https://arxiv.org/abs/1907.04749)

## 评测

## 性能基准

| 库 | 过滤器 | 构建(万ops/s) | 查询(万ops/s) | 内存占用 | 对比 |
| --- | --- | --- | --- | --- | --- |
| xorf | Bf8 | 3985.84 | 52562.99 | 116.04 KB | - |
| xorf | BinaryFuse16 | 4036.86 | 48915.75 | 232.04 KB | - |
| xorf | BinaryFuse32 | 3931.59 | 48225.88 | 464.04 KB | - |
| jdb | Bf16 | 4599.84 | 76251.81 | 232.04 KB | - |
| jdb | Bf32 | 4767.73 | 66797.11 | 464.04 KB | - |
| jdb | Bf8 | 4742.57 | 86450.03 | 116.04 KB | - |

## 准确率

| 库 | 过滤器 | 假阳率 | 假阴率 |
| --- | --- | --- | --- |
| xorf | Bf8 | 0.39252% | 0 |
| xorf | BinaryFuse16 | 0.00157% | 0 |
| xorf | BinaryFuse32 | 0.00000% | 0 |
| jdb | Bf16 | 0.00144% | 0 |
| jdb | Bf32 | 0.00000% | 0 |
| jdb | Bf8 | 0.39059% | 0 |

---

## 关于

本项目为 [js0.site ⋅ 重构互联网计划](https://js0.site) 的开源组件。

我们正在以组件化的方式重新定义互联网的开发范式，欢迎关注：

* [谷歌邮件列表](https://groups.google.com/g/js0-site)
* [js0site.bsky.social](https://bsky.app/profile/js0site.bsky.social)
