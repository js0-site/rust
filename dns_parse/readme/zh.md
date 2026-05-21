# dns_parse : DNS 消息构建与解析

轻量级 DNS 消息构建和解析库，供 [idoq](https://crates.io/crates/idoq) (DoQ) 和 [idot](https://crates.io/crates/idot) (DoT) 使用。

## 特性

- 构建带 EDNS 的 DNS 查询消息
- 解析 DNS 响应消息
- 支持 A、AAAA、MX、TXT、NS、CNAME、PTR、SRV 记录
- DNS 名称压缩（指针）处理
- 基于 `bytes` 的零拷贝解析

## 安装

```toml
[dependencies]
dns_parse = "0.1"
```

## 使用

### 构建 DNS 查询

```rust
use dns_parse::build;

// 构建查询：消息 ID、域名、查询类型
let msg = build(0x1234, "example.com", 1); // A 记录
let msg = build(0, "example.com", 28);     // AAAA 记录 (DoQ 要求 ID=0)
```

### 解析 DNS 响应

```rust
use dns_parse::parse;
use bytes::Bytes;

let response: Bytes = /* DNS 响应数据 */;
match parse(response) {
  Ok(answers) => {
    for a in answers {
      println!("{} {} TTL={}", a.name, a.val, a.ttl);
    }
  }
  Err(e) => eprintln!("解析错误: {e}"),
}
```

## API 参考

### 函数

#### `build`

构建 DNS 查询消息。

```rust
pub fn build(id: u16, domain: &str, qtype: u16) -> Bytes
```

- `id`: 消息 ID（DoQ 按 RFC 9250 要求为 0，DoT 可随机）
- `domain`: 查询域名
- `qtype`: 查询类型（1=A, 28=AAAA, 15=MX 等）

返回带 EDNS OPT 记录（4096 字节 UDP 负载）的 DNS 查询消息。

#### `parse`

解析 DNS 响应消息。

```rust
pub fn parse(data: Bytes) -> Result<Vec<Answer>>
```

返回解析后的应答记录。NXDOMAIN 或错误响应返回空向量。

### 类型

#### `Answer`（来自 idns）

```rust
pub struct Answer {
  pub name: String,   // 记录名称
  pub val: String,    // 记录值（格式化后）
  pub type_id: u16,   // 记录类型
  pub ttl: u32,       // 生存时间
}
```

### 错误类型

- `ResponseTooShort` - 响应小于 12 字节
- `IncompleteData` - 记录数据截断
- `NameOutOfBounds` - 名称超出消息边界
- `PointerOutOfBounds` - 无效压缩指针

## 记录值格式

| 类型         | 格式                        |
| ------------ | --------------------------- |
| A (1)        | `192.0.2.1`                 |
| AAAA (28)    | `2001:db8::1`               |
| MX (15)      | `10 mail.example.com`       |
| TXT (16)     | `v=spf1 include:...`        |
| NS/CNAME/PTR | `ns1.example.com`           |
| SRV (33)     | `10 5 5060 sip.example.com` |
| 其他         | 十六进制编码                |

## 技术栈

| 组件     | 库        |
| -------- | --------- |
| 缓冲     | bytes     |
| 错误     | thiserror |
| 十六进制 | hex       |
