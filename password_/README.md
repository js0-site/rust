[English](#en) | [中文](#zh)

---

<a id="en"></a>

# password_ : Secure Password Hashing Made Simple

A lightweight and secure password hashing library based on Argon2.

## Table of Contents

- [Introduction](#introduction)
- [Usage](#usage)
- [Design](#design)
- [Tech Stack](#tech-stack)
- [Directory Structure](#directory-structure)
- [API Reference](#api-reference)
- [History](#history)

## Introduction

`password_` provides a simplified interface for hashing and verifying passwords using the state-of-the-art Argon2 algorithm. It abstracts away complex configuration, offering secure defaults for immediate use.

## Usage

See `tests/main.rs` for a complete demonstration.

```rust
use aok::{OK, Void};
use log::info;

fn main() -> Void {
  let password = "test";
  // Generate a random salt and hash the password
  let (salt, hash) = password_::hash(password);
  
  info!("{salt:?} {hash:?}");

  // Verify the password against the salt and hash
  assert!(password_::verify(password, &salt, &hash));
  
  OK
}
```

## Design

The library uses a static configuration for Argon2 to ensure consistency and security:
- **Algorithm**: Argon2id (hybrid version, resistant to GPU and side-channel attacks).
- **Version**: 0x13.
- **Memory**: 64 MB (65536 KB).
- **Iterations**: 3.
- **Parallelism**: 1.
- **Output Length**: 32 bytes.

The `hash` function generates a random 16-byte salt and computes the 32-byte hash. The `verify` function re-computes the hash using the provided salt and compares it with the stored hash.

## Tech Stack

- **Language**: Rust
- **Core Algorithm**: `argon2` crate
- **Randomness**: `rand` crate
- **Initialization**: `static_init` crate

## Directory Structure

```
.
├── src/
│   └── lib.rs       # Core logic and API definitions
├── tests/
│   └── main.rs      # Usage demonstration and tests
├── readme/          # Documentation
└── Cargo.toml       # Project configuration
```

## API Reference

The library exports the following from `lib.rs`:

### Types

- `SALT`: Alias for `[u8; 16]`.
- `HASH`: Alias for `[u8; 32]`.

### Functions

- `fn hash(password: impl AsRef<[u8]>) -> (SALT, HASH)`
  Generates a random salt and returns the (salt, hash) tuple.

- `fn hash_with_salt(password: impl AsRef<[u8]>, salt: impl AsRef<[u8]>) -> HASH`
  Computes the hash for a given password and salt.

- `fn verify(password: impl AsRef<[u8]>, salt: impl AsRef<[u8]>, hash: impl AsRef<[u8]>) -> bool`
  Verifies if the password matches the provided salt and hash.

## History

In 2013, the Password Hashing Competition (PHC) was launched to find a successor to aging algorithms like PBKDF2 and bcrypt, which were becoming vulnerable to GPU-based attacks. After two years of rigorous analysis, **Argon2** was selected as the winner in July 2015. Designed by Alex Biryukov, Daniel Dinu, and Dmitry Khovratovich, Argon2 introduced memory-hardness properties that make it prohibitively expensive to crack using specialized hardware, setting a new standard for password security.

---

## About

This project is an open-source component of [js0.site ⋅ Refactoring the Internet Plan](https://js0.site).

We are redefining the development paradigm of the Internet in a componentized way. Welcome to follow us:

* [Google Group](https://groups.google.com/g/js0-site)
* [js0site.bsky.social](https://bsky.app/profile/js0site.bsky.social)

---

<a id="zh"></a>

# password_ : 简单安全的密码哈希库

基于 Argon2 算法的轻量级安全密码哈希库。

## 目录

- [简介](#简介)
- [使用演示](#使用演示)
- [设计思路](#设计思路)
- [技术堆栈](#技术堆栈)
- [目录结构](#目录结构)
- [API 参考](#api-参考)
- [历史轶事](#历史轶事)

## 简介

`password_` 为密码哈希和验证提供了简化的接口，底层采用先进的 Argon2 算法。它封装了复杂的配置参数，提供安全的默认设置，开箱即用。

## 使用演示

完整演示请参考 `tests/main.rs`。

```rust
use aok::{OK, Void};
use log::info;

fn main() -> Void {
  let password = "test";
  // 生成随机盐并计算哈希
  let (salt, hash) = password_::hash(password);
  
  info!("{salt:?} {hash:?}");

  // 验证密码是否匹配
  assert!(password_::verify(password, &salt, &hash));
  
  OK
}
```

## 设计思路

本库采用静态的 Argon2 配置以确保一致性和安全性：
- **算法**: Argon2id (混合版本，抗 GPU 和侧信道攻击)。
- **版本**: 0x13。
- **内存**: 64 MB (65536 KB)。
- **迭代次数**: 3。
- **并行度**: 1。
- **输出长度**: 32 字节。

`hash` 函数生成一个随机的 16 字节盐值并计算 32 字节哈希值。`verify` 函数使用提供的盐值重新计算哈希，并与存储的哈希值进行比对。

## 技术堆栈

- **语言**: Rust
- **核心算法**: `argon2` crate
- **随机数**: `rand` crate
- **初始化**: `static_init` crate

## 目录结构

```
.
├── src/
│   └── lib.rs       # 核心逻辑与 API 定义
├── tests/
│   └── main.rs      # 使用演示与测试
├── readme/          # 文档
└── Cargo.toml       # 项目配置
```

## API 参考

`lib.rs` 导出以下内容：

### 类型

- `SALT`: `[u8; 16]` 的别名。
- `HASH`: `[u8; 32]` 的别名。

### 函数

- `fn hash(password: impl AsRef<[u8]>) -> (SALT, HASH)`
  生成随机盐并返回 (salt, hash) 元组。

- `fn hash_with_salt(password: impl AsRef<[u8]>, salt: impl AsRef<[u8]>) -> HASH`
  使用给定的盐计算密码哈希。

- `fn verify(password: impl AsRef<[u8]>, salt: impl AsRef<[u8]>, hash: impl AsRef<[u8]>) -> bool`
  验证密码是否与提供的盐和哈希匹配。

## 历史轶事

2013 年，为了寻找 PBKDF2 和 bcrypt 等老旧算法的继任者（这些算法在 GPU 攻击面前日益脆弱），密码哈希竞赛 (PHC) 正式启动。经过两年的严格分析，**Argon2** 于 2015 年 7 月脱颖而出夺得桂冠。Argon2 由 Alex Biryukov、Daniel Dinu 和 Dmitry Khovratovich 设计，引入了内存硬化 (memory-hardness) 特性，使得使用专用硬件破解密码的成本极其高昂，从而树立了密码安全的新标准。

---

## 关于

本项目为 [js0.site ⋅ 重构互联网计划](https://js0.site) 的开源组件。

我们正在以组件化的方式重新定义互联网的开发范式，欢迎关注：

* [谷歌邮件列表](https://groups.google.com/g/js0-site)
* [js0site.bsky.social](https://bsky.app/profile/js0site.bsky.social)
