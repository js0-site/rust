# sk_dkim : 基于密钥的确定性 DKIM 密钥生成

## 目录

- [简介](#简介)
- [为什么是 RSA？](#为什么是-rsaed25519-的悲惨现状)
- [使用方法](#使用方法)
- [设计思路](#设计思路)
- [API 参考](#api-参考)
- [技术栈](#技术栈)
- [目录结构](#目录结构)

## 简介

`sk_dkim` 是一个 Rust 库，用于确定性地生成 **DomainKeys Identified Mail (DKIM)** 密钥和 DNS TXT 记录。无需为每个域名单独管理和存储私钥文件，只需使用一个主密钥种子 (Secret Key)，结合域名和选择器 (Selector)，即可实时派生出所需的 **RSA 2048** 密钥。

这种方法极大地简化了密钥管理，特别是对于需要为大量域名提供 DKIM 签名的服务，彻底消除了对私钥状态存储的需求。

## 为什么是 RSA？(Ed25519 的悲惨现状)

从技术角度来看，**Ed25519** 在 DKIM 应用中几乎全方位优于 RSA：

- **体积小**：Ed25519 公钥仅 32 字节，可以轻松放入单条 DNS TXT 记录中。而 RSA 2048 公钥非常庞大，通常需要分割成多个字符串才能存入 TXT 记录。
- **高性能**：Ed25519 的签名和验证速度极快。
- **安全性**：Ed25519 在较小的密钥尺寸下提供了极高的安全性。

**RFC 8463** 早在 **2018 年** 就已发布，其中明确规定验证者 **必须 (MUST)** 实现 Ed25519 验证。然而，到了 2024/2025 年，电子邮件生态系统的现状却是行业巨头们集体违规的失望展示：

- **Gmail**: 支持情况极不稳定。虽然声称支持，但对于使用 Ed25519 签名的入站邮件，验证经常失败。
- **Outlook / Microsoft 365**: 它们 **始终无法** 验证 Ed25519 签名。如果你只用 Ed25519 签名发送邮件给 Outlook/Hotmail 用户，邮件很可能会被视为未签名或验证失败。
- **Yahoo**: 与微软类似，Yahoo 邮箱通常也无法验证入站的 Ed25519 DKIM 签名。

尽管该标准已经发布超过 6 年，这些主要服务商实际上完全无视了验证方面的 "MUST implement" 条款。虽然一些专家建议 "双重签名"（同时使用 RSA 和 Ed25519 签名），但这会显著增加复杂性和邮件头的臃肿程度。

正是由于这种广泛的缺乏支持，`sk_dkim` 被迫从 Ed25519 切换回 **RSA 2048**。为了确保邮件在现实世界中的可达性，我们不得不牺牲优雅和效率。这是技术的倒退，但在当前环境下却是无奈且必须的选择。

## 使用方法

在 `Cargo.toml` 中添加 `sk_dkim`：

```toml
[dependencies]
sk_dkim = { version = "0.1.5", features = ["pk"] }
```

> 注意：生成格式化的 TXT 记录字符串需要开启 `pk` 特性。

### 演示代码

```rust
use sk_dkim::Sk;

fn main() {
    // 主密钥种子 (请务必妥善保管)
    let secret_seed = "your_secret_seed_string";

    // 使用种子初始化生成器
    let sk = Sk::new(secret_seed);

    let selector = "default";
    let domain = "example.com";

    // 为特定域名和选择器生成 DKIM 结构
    let dkim = sk.dkim(selector, domain);

    // 获取 DNS TXT 记录值
    // 输出格式: v=DKIM1; k=rsa; p=...
    println!("DKIM Record: {}", dkim.txt());
}
```

## 设计思路

`sk_dkim` 的核心理念是**确定性**。

1.  **初始化**：`Sk` 结构体使用基础的主密钥种子进行初始化。该种子用于初始化一个 `BLAKE3` 哈希器。
2.  **派生**：调用 `dkim(selector, domain)` 时，克隆哈希器并利用 `selector` 和 `domain` 更新哈希状态。
3.  **密钥生成**：最终的哈希摘要作为种子，用于初始化 **ChaCha20Rng**，进而生成确定性的 **RSA 2048** 密钥对。
4.  **输出**：公钥部分经过 Base64 编码 (SPKI 格式)，并格式化为标准的 DKIM TXT 记录。

此流程确保只要主密钥种子保持不变，对于给定的域名，生成的 DKIM 密钥将始终一致。

## API 参考

### `struct Sk`

密钥生成的主要入口点。

- **`Sk::new(sk: impl AsRef<[u8]>) -> Self`**
  使用提供的主密钥种子创建一个新的 `Sk` 实例。

- **`Sk::dkim(&self, selector: impl AsRef<str>, domain: impl AsRef<str>) -> Dkim`**
  为指定的选择器 (`selector`) 和域名 (`domain`) 派生一个 `Dkim` 实例。

### `struct Dkim`

表示生成的 DKIM 密钥对。

- **`pub sk: rsa::RsaPrivateKey`**
  底层的 RSA 私钥。

- **`Dkim::txt(&self) -> String`**
  _(需要 `pk` 特性)_
  返回格式化的 DKIM DNS TXT 记录字符串 (例如 `v=DKIM1; k=rsa; p=...`)。

## 技术栈

- **Rust**: 核心开发语言。
- **rsa**: 纯 Rust 实现的 RSA 算法。
- **rand_chacha**: 加密安全的随机数生成器 (ChaCha20)，用于从种子生成确定性密钥。
- **blake3**: 用于确定性密钥派生的加密哈希算法。
- **base64**: 用于 DNS 记录的公钥编码。

## 目录结构

```
.
├── Cargo.toml      # 项目配置与依赖
├── readme/         # 文档目录
│   ├── en.md       # 英文说明文档
│   └── zh.md       # 中文说明文档
├── src/            # 源代码
│   └── lib.rs      # 库入口与实现
├── tests/          # 集成测试
│   └── main.rs     # 使用演示与测试
└── test.sh         # 测试执行脚本
```
