# password\_ : 简单安全的密码哈希库

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
