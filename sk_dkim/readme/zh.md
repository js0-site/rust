# sk_dkim : 基于密钥的确定性 DKIM 密钥生成

## 目录

- [简介](#简介)
- [使用方法](#使用方法)
- [设计思路](#设计思路)
- [API 参考](#api-参考)
- [技术栈](#技术栈)
- [目录结构](#目录结构)
- [历史与趣闻](#历史与趣闻)

## 简介

`sk_dkim` 是一个 Rust 库，用于确定性地生成 **DomainKeys Identified Mail (DKIM)** 密钥和 DNS TXT 记录。无需为每个域名单独管理和存储私钥文件，只需使用一个主密钥种子 (Secret Key)，结合域名和选择器 (Selector)，即可实时派生出所需的 Ed25519 密钥。

这种方法极大地简化了密钥管理，特别是对于需要为大量域名提供 DKIM 签名的服务，彻底消除了对私钥状态存储的需求。

## 使用方法

在 `Cargo.toml` 中添加 `sk_dkim`：

```toml
[dependencies]
sk_dkim = { version = "0.1.1", features = ["pk"] }
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
    // 输出格式: v=DKIM1;k=ed25519;p=...
    println!("DKIM Record: {}", dkim.txt());
}
```

## 设计思路

`sk_dkim` 的核心理念是**确定性**。

1.  **初始化**：`Sk` 结构体使用基础的主密钥种子进行初始化。该种子用于初始化一个 `BLAKE3` 哈希器。
2.  **派生**：调用 `dkim(selector, domain)` 时，克隆哈希器并利用 `selector` 和 `domain` 更新哈希状态。
3.  **密钥生成**：最终的哈希摘要作为种子，用于生成 **Ed25519** 签名密钥。
4.  **输出**：公钥部分经过 Base64 编码，并格式化为标准的 DKIM TXT 记录。

此流程确保只要主密钥种子保持不变，对于给定的域名，生成的 DKIM 密钥将始终一致。

## API 参考

### `struct Sk`

密钥生成的主要入口点。

*   **`Sk::new(sk: impl AsRef<[u8]>) -> Self`**
    使用提供的主密钥种子创建一个新的 `Sk` 实例。

*   **`Sk::dkim(&self, selector: impl AsRef<str>, domain: impl AsRef<str>) -> Dkim`**
    为指定的选择器 (`selector`) 和域名 (`domain`) 派生一个 `Dkim` 实例。

### `struct Dkim`

表示生成的 DKIM 密钥对。

*   **`pub sk: ed25519_dalek::SigningKey`**
    底层的 Ed25519 签名密钥。

*   **`Dkim::txt(&self) -> String`**
    *(需要 `pk` 特性)*
    返回格式化的 DKIM DNS TXT 记录字符串 (例如 `v=DKIM1;k=ed25519;p=...`)。

## 技术栈

*   **Rust**: 核心开发语言。
*   **ed25519-dalek**: 高效且安全的 Ed25519 密钥生成与签名。
*   **blake3**: 用于确定性密钥派生的加密哈希算法。
*   **base64**: 用于 DNS 记录的公钥编码。

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

## 历史与趣闻

**DKIM (DomainKeys Identified Mail)** 诞生于 2007 年，由 Yahoo! 的 **DomainKeys** 和 Cisco 的 **Identified Internet Mail (IIM)** 两个独立的邮件认证协议合并而来。它于 2011 年正式成为 IETF 标准 (RFC 6376)。

本项目使用的 **Ed25519** 算法，是由 Daniel J. Bernstein 团队在 2011 年推出的高性能公钥签名系统。它基于 **Curve25519**，相比 RSA 等老旧算法，在安全性和性能上都有显著优势。

这两项技术的交汇点发生在 2018 年发布的 **RFC 8463**，该标准正式将 Ed25519 引入 DKIM 签名。这是一个重要的里程碑，因为 Ed25519 密钥比同等强度的 RSA 密钥短得多，这使得它们更容易放入 DNS TXT 记录中，而不会触及长度限制。