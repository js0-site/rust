[English](#en) | [中文](#zh)

---

<a id="en"></a>

# vb : Efficient compression for integer sequences

## Table of Contents
- [Introduction](#introduction)
- [Features](#features)
- [Usage](#usage)
  - [Basic Encoding](#basic-encoding)
  - [Differential Encoding](#differential-encoding)
- [API Reference](#api-reference)
- [Design](#design)
- [Tech Stack](#tech-stack)
- [Directory Structure](#directory-structure)
- [History](#history)

## Introduction
`vb` is a lightweight Rust library for **Variable Byte (VByte)** encoding. It provides an efficient way to compress `u64` integers and lists of integers. By using fewer bytes for smaller numbers, it significantly reduces storage requirements for integer-heavy data.

Additionally, the library supports **Differential Encoding (Delta Encoding)**, which is particularly effective for compressing strictly increasing sequences (e.g., sorted IDs, timestamps) by storing only the differences between consecutive values.

## Features
- **Variable Byte Encoding**: Compresses `u64` integers into a variable-length byte sequence.
- **Differential Encoding**: Optimizes storage for strictly increasing sequences (requires `diff` feature).
- **Zero-Copy Decoding**: Efficient decoding directly from byte slices.
- **Simple API**: Easy-to-use functions for single values and lists.
- **Error Handling**: Robust error reporting using `thiserror`.

## Usage

Add `vb` to your `Cargo.toml`:

```toml
[dependencies]
vb = "0.1.8"
# For differential encoding support
# vb = { version = "0.1.8", features = ["diff"] }
```

### Basic Encoding

```rust
use vb::{e_li, d_li};

fn main() {
    let list = vec![0, 1, 127, 128, 300, 16383, 16384];

    // Encode list
    let encoded = e_li(&list);
    println!("Encoded length: {} bytes", encoded.len());

    // Decode list
    let decoded = d_li(&encoded).unwrap();
    assert_eq!(list, decoded);
}
```

### Differential Encoding

Useful for sorted sequences like timestamps or primary keys.

```rust
#[cfg(feature = "diff")]
use vb::{e_diff, d_diff};

#[cfg(feature = "diff")]
fn main() {
    // Strictly increasing sequence
    let list = vec![10000, 10001, 10002, 10003, 10004];

    // Encode using differential encoding
    let encoded = e_diff(&list);

    // Decode back to original sequence
    let decoded = d_diff(&encoded).unwrap();
    assert_eq!(list, decoded);
}
```

## API Reference

The library exports the following key functions:

- **`d(input: impl AsRef<[u8]>) -> Result<(u64, usize)>`**
  Decodes a single variable-byte encoded integer. Returns the value and bytes consumed.

- **`e(value: u64, bytes: &mut Vec<u8>)`**
  Encodes a single `u64` into variable-byte format and appends to the buffer.


- **`e_li(li: impl AsRef<[u64]>) -> Vec<u8>`**
  Encodes a list of `u64` integers into a byte vector.

- **`d_li(data: impl AsRef<[u8]>) -> Result<Vec<u64>>`**
  Decodes a byte vector back into a list of `u64` integers.

- **`e_diff(li: impl AsRef<[u64]>) -> Vec<u8>`** *(feature: `diff`)*
  Encodes a strictly increasing sequence using differential encoding combined with VByte.

- **`d_diff(vs: impl AsRef<[u8]>) -> Result<Vec<u64>>`** *(feature: `diff`)*
  Decodes a differentially encoded sequence.

## Design

The VByte format uses 7 bits of each byte for data and the Most Significant Bit (MSB) as a continuation flag.
- **MSB = 0**: The last byte of the integer.
- **MSB = 1**: More bytes follow.

For differential encoding (`e_diff`), the library first calculates the difference between adjacent elements ($x_i - x_{i-1}$) and then encodes these smaller differences using VByte. This results in significant compression for dense, increasing sequences.

## Tech Stack
- **Rust**: Core language.
- **thiserror**: For ergonomic error handling.

## Directory Structure

```
.
├── Cargo.toml      # Project configuration
├── readme          # Documentation
│   ├── en.md       # English README
│   └── zh.md       # Chinese README
├── src
│   └── lib.rs      # Source code
└── tests
    └── main.rs     # Integration tests
```

## History

Variable Byte encoding (also known as **VByte**, **Varint**, or **LEB128**) has a rich history in computer science.

- **MIDI Standard**: One of the earliest widespread uses was in the MIDI file format as "Variable-Length Quantity" (VLQ) to save space in music files.
- **Search Engines**: In the late 90s, search engines like Google used VByte to compress inverted indexes (lists of document IDs), balancing high compression ratios with very fast decoding speeds.
- **DWARF**: The DWARF debugging data format uses a variant called LEB128 (Little Endian Base 128).
- **Protocol Buffers**: Google's Protocol Buffers heavily rely on "Varints" for efficient data serialization.

This simple yet powerful technique remains a cornerstone of data compression in systems where both space and speed matter.

---

## About

This project is an open-source component of [js0.site ⋅ Refactoring the Internet Plan](https://js0.site).

We are redefining the development paradigm of the Internet in a componentized way. Welcome to follow us:

* [Google Group](https://groups.google.com/g/js0-site)
* [js0site.bsky.social](https://bsky.app/profile/js0site.bsky.social)

---

<a id="zh"></a>

# vb : 高效的整数序列压缩方案

## 目录
- [简介](#简介)
- [功能特性](#功能特性)
- [使用指南](#使用指南)
  - [基础编码](#基础编码)
  - [差分编码](#差分编码)
- [API 参考](#api-参考)
- [设计思路](#设计思路)
- [技术栈](#技术栈)
- [目录结构](#目录结构)
- [历史轶事](#历史轶事)

## 简介
`vb` 是一个轻量级的 Rust 库，用于实现 **变长字节编码 (Variable Byte Encoding)**。它提供了一种高效的方法来压缩 `u64` 整数及其列表。通过对较小的数字使用更少的字节，它能显著减少整数密集型数据的存储需求。

此外，该库支持 **差分编码 (Delta Encoding)**，通过仅存储连续值之间的差值，对严格递增的序列（如排序后的 ID、时间戳）具有极佳的压缩效果。

## 功能特性
- **变长字节编码**：将 `u64` 整数压缩为变长字节序列。
- **差分编码**：优化严格递增序列的存储（需开启 `diff` 特性）。
- **零拷贝解码**：直接从字节切片高效解码。
- **简洁 API**：提供针对单值和列表的易用函数。
- **错误处理**：使用 `thiserror` 提供健壮的错误报告。

## 使用指南

在 `Cargo.toml` 中添加 `vb`：

```toml
[dependencies]
vb = "0.1.8"
# 如需差分编码支持
# vb = { version = "0.1.8", features = ["diff"] }
```

### 基础编码

```rust
use vb::{e_li, d_li};

fn main() {
    let list = vec![0, 1, 127, 128, 300, 16383, 16384];

    // 编码列表
    let encoded = e_li(&list);
    println!("编码后长度: {} 字节", encoded.len());

    // 解码列表
    let decoded = d_li(&encoded).unwrap();
    assert_eq!(list, decoded);
}
```

### 差分编码

适用于时间戳或主键等排序序列。

```rust
#[cfg(feature = "diff")]
use vb::{e_diff, d_diff};

#[cfg(feature = "diff")]
fn main() {
    // 严格递增序列
    let list = vec![10000, 10001, 10002, 10003, 10004];

    // 使用差分编码进行压缩
    let encoded = e_diff(&list);

    // 解码回原始序列
    let decoded = d_diff(&encoded).unwrap();
    assert_eq!(list, decoded);
}
```

## API 参考

本库导出以下核心函数：

- **`d(input: impl AsRef<[u8]>) -> Result<(u64, usize)>`**
  解码单个变长编码整数。返回数值及消耗的字节数。

- **`e(value: u64, bytes: &mut Vec<u8>)`**
  将单个 `u64` 编码为变长格式并追加到缓冲区。


- **`e_li(li: impl AsRef<[u64]>) -> Vec<u8>`**
  将 `u64` 整数列表编码为字节向量。

- **`d_li(data: impl AsRef<[u8]>) -> Result<Vec<u64>>`**
  将字节向量解码回 `u64` 整数列表。

- **`e_diff(li: impl AsRef<[u64]>) -> Vec<u8>`** *(特性: `diff`)*
  结合差分编码与变长编码，压缩严格递增序列。

- **`d_diff(vs: impl AsRef<[u8]>) -> Result<Vec<u64>>`** *(特性: `diff`)*
  解码差分编码序列。

## 设计思路

VByte 格式使用每个字节的低 7 位存储数据，最高有效位 (MSB) 作为延续标志。
- **MSB = 0**：表示这是整数的最后一个字节。
- **MSB = 1**：表示后续还有更多字节。

对于差分编码 (`e_diff`)，库首先计算相邻元素之间的差值 ($x_i - x_{i-1}$)，然后使用 VByte 对这些较小的差值进行编码。这种方法对于密集的递增序列能产生显著的压缩效果。

## 技术栈
- **Rust**：核心开发语言。
- **thiserror**：用于符合人体工程学的错误处理。

## 目录结构

```
.
├── Cargo.toml      # 项目配置
├── readme          # 文档目录
│   ├── en.md       # 英文说明
│   └── zh.md       # 中文说明
├── src
│   └── lib.rs      # 源代码
└── tests
    └── main.rs     # 集成测试
```

## 历史轶事

变长字节编码（也称为 **VByte**、**Varint** 或 **LEB128**）在计算机科学史上拥有悠久的历史。

- **MIDI 标准**：最早的广泛应用之一是在 MIDI 文件格式中，被称为“变长数量”(VLQ)，旨在节省音乐文件的存储空间。
- **搜索引擎**：90 年代末，Google 等搜索引擎利用 VByte 压缩倒排索引（文档 ID 列表），在保持高压缩率的同时实现了极快的解码速度。
- **DWARF**：DWARF 调试数据格式使用了一种名为 LEB128 (Little Endian Base 128) 的变体。
- **Protocol Buffers**：Google 的 Protocol Buffers 序列化协议也严重依赖 "Varints" 来实现高效的数据传输。

这项简单而强大的技术，在对空间和速度都有要求的系统中，至今仍是数据压缩的基石。

---

## 关于

本项目为 [js0.site ⋅ 重构互联网计划](https://js0.site) 的开源组件。

我们正在以组件化的方式重新定义互联网的开发范式，欢迎关注：

* [谷歌邮件列表](https://groups.google.com/g/js0-site)
* [js0site.bsky.social](https://bsky.app/profile/js0site.bsky.social)
