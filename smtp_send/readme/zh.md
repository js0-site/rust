# smtp_send

支持 DKIM 签名、MX 查询和自动退信处理的 SMTP 邮件发送库。

## 功能

- **DKIM 签名**: RSA-SHA256，RFC 6376 头部保护（N+1 签名）
- **MX 查询**: 通过 `idot` 使用 DNS-over-QUIC
- **并发发送**: 按域名分组并行投递
- **故障转移**: 失败时尝试多个 MX 服务器
- **退信处理**: 自动生成符合 RFC 5321 的拒信报告
- **循环防护**: 检查 `Received` 头数量（最大 30）及 `Auto-Submitted`/`Return-Path` 头
- **DKIM 缓存**: 签名器 600 秒 TTL 缓存

## 使用

```rust
use smtp_send::Send;
use mail_struct::Mail;

#[tokio::main]
async fn main() {
    // 加载 DKIM 私钥
    let sk = std::fs::read("private.key").unwrap();

    // 创建发送器，指定 DKIM 选择器
    let sender = Send::new("default", &sk);

    // 构建邮件
    let mut mail = Mail::new(
        "sender@example.com",
        ["recipient@example.com"],
        b"Subject: Test\r\n\r\nHello".to_vec(),
    ).unwrap();

    // 发送邮件
    let result = sender.send(&mut mail).await;

    println!("成功: {}, 错误: {}", result.success, result.error_li.len());
}
```

## API

### `Send`

```rust
pub struct Send {
    pub selector: String,  // DKIM 选择器
    pub sk: Sk,            // DKIM 私钥
}

impl Send {
    // 创建发送器
    fn new(selector: impl Into<String>, sk: impl AsRef<[u8]>) -> Self;

    // 发送邮件，自动添加 Received 头和 DKIM 签名
    async fn send(&self, mail: &mut Mail) -> SendResult;
}
```

### `SendResult`

```rust
pub struct SendResult {
    pub success: usize,        // 成功投递数
    pub error_li: Vec<Error>,  // 错误列表
}
```

### `Error`

| 变体                          | 说明                  |
| ----------------------------- | --------------------- |
| `DkimInit(host)`              | DKIM 签名器初始化失败 |
| `DnsResolveFailed(host, err)` | DNS 解析失败          |
| `MxIsEmpty(host)`             | 未找到 MX 记录        |
| `Reject(Reject)`              | 服务器拒绝            |
| `SendErr(SendErr)`            | 发送失败              |
| `SmtpAllFailed(host, err)`    | 所有 MX 服务器均失败  |
| `TooManyReceived(sender)`     | Received 头超过 30 个 |

### 独立函数

```rust
// 不添加 Received 头直接发送
pub async fn send(mail: &Mail, signer: Option<&Signer>) -> SendResult;

// 创建 DKIM 签名器（带缓存），失败返回 None
pub fn signer(selector: &str, host: &str, sk: &Sk) -> Option<RefVal<String, Signer>>;

// 检查 Received 头是否超限
pub fn recv_overflow(body: &[u8]) -> bool;

// 添加 Received 头
pub fn add_received(body: &mut Vec<u8>, from: &str, by: &str);

pub const MAX_RECEIVED: usize = 30;
```

## 目录结构

```
src/
├── lib.rs      # 入口，Send 结构体，并发发送
├── dkim.rs     # DKIM 签名器及缓存
├── error.rs    # 错误类型
├── parse.rs    # Received 头解析
├── send.rs     # 单域名发送逻辑
├── smtp.rs     # SMTP 连接封装
└── reject/     # 退信生成
    ├── mod.rs
    ├── create_tar_zstd.rs
    ├── encode_mail.rs
    └── reject_mail.rs
```

## 技术栈

- `mail_send` - SMTP 协议
- `idot` / `idns` - DNS-over-QUIC MX 查询
- `sk_dkim` - DKIM 密钥管理
- `mail-parser` - 邮件解析（用于退信生成）
- `papaya` - DKIM 签名器缓存
- `async-scoped` - 作用域异步任务，用于并发发送
