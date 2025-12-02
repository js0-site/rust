# SMTP发送问题解答 / SMTP Sending Q&A

## 问题1: 这里用了加密通讯吗？/ Question 1: Is encryption used?

### 原始代码分析 / Original Code Analysis

**答案：是的，但是是机会性的（Opportunistic）**

```rust
SmtpClientBuilder::new(&mx.server, 25)
  .implicit_tls(false)  // ← 这个配置启用了STARTTLS
  .connect()
```

### 详细说明 / Detailed Explanation

当你使用 `implicit_tls(false)` 时，`mail-send` 库会：

1. **首先建立明文连接**到端口25
2. **发送 EHLO 命令**查询服务器能力
3. **如果服务器支持 STARTTLS**：
   - 发送 `STARTTLS` 命令
   - 升级连接到TLS加密
   - 继续加密通讯
4. **如果服务器不支持 STARTTLS**：
   - 继续使用明文连接
   - 不会报错，邮件仍然会发送

这就是所谓的**机会性TLS（Opportunistic TLS）**。

### 不同配置的对比 / Configuration Comparison

| 配置 | 端口 | implicit_tls | 行为 |
|------|------|--------------|------|
| **当前配置** | 25 | false | 机会性STARTTLS（尝试加密，失败则明文） |
| 隐式TLS | 465 | true | 立即使用TLS（SMTPS） |
| 提交端口 | 587 | false | STARTTLS（通常需要认证） |
| 明文（不推荐） | 25 | - | 完全不加密 |

---

## 问题2: 怎么修复上面的错误？/ Question 2: How to fix the errors?

### 错误类型分析 / Error Type Analysis

你遇到了**两种不同的错误**：

#### 错误A: Google邮箱不存在 (550 5.1.1)
```
The email account that you tried to reach does not exist.
```

**这不是你的服务器问题！** 这是收件人邮箱的问题。

**解决方案：**
1. ✅ 检查收件人邮箱地址拼写
2. ✅ 确认收件人邮箱确实存在
3. ✅ 不要发送到不存在的邮箱

#### 错误B: Cloudflare反向DNS拒绝 (550 0.0.0)
```
Sender IP reverse lookup rejected (2605:a141:2288:4651::)
```

**这是你的服务器配置问题！** 你的IP没有正确的PTR记录。

**解决方案：**

### 步骤1: 检查当前PTR记录

```bash
# 检查你的IPv6地址的PTR记录
dig -x 2605:a141:2288:4651::

# 应该返回类似这样的结果：
# 2605:a141:2288:4651:: PTR mail.yourdomain.com
```

### 步骤2: 配置PTR记录

你需要联系你的服务器托管商配置PTR记录：

**正确的配置：**
```
IP地址: 2605:a141:2288:4651::
  ↓ PTR记录
域名: mail.yourdomain.com
  ↓ AAAA记录
IP地址: 2605:a141:2288:4651::
```

**必须满足：**
- PTR记录：`2605:a141:2288:4651::` → `mail.yourdomain.com`
- AAAA记录：`mail.yourdomain.com` → `2605:a141:2288:4651::`
- 两者必须匹配！

### 步骤3: 配置完整的邮件DNS记录

```bash
# 1. A/AAAA记录
mail.yourdomain.com.  IN  AAAA  2605:a141:2288:4651::

# 2. MX记录
yourdomain.com.  IN  MX  10 mail.yourdomain.com.

# 3. SPF记录
yourdomain.com.  IN  TXT  "v=spf1 ip6:2605:a141:2288:4651:: -all"

# 4. PTR记录（需要联系托管商配置）
# 2605:a141:2288:4651:: → mail.yourdomain.com
```

### 步骤4: 验证配置

```bash
# 检查MX记录
dig yourdomain.com MX

# 检查A/AAAA记录
dig mail.yourdomain.com AAAA

# 检查PTR记录
dig -x 2605:a141:2288:4651::

# 检查SPF记录
dig yourdomain.com TXT
```

---

## 代码改进建议 / Code Improvement Recommendations

### 选项1: 使用当前改进的代码（推荐）

当前的 `src/send.rs` 已经更新为：
- ✅ 支持机会性STARTTLS
- ✅ 更详细的错误日志
- ✅ 双语错误提示
- ✅ 错误类型分析

### 选项2: 使用新的配置化版本

我创建了 `src/send_v2.rs`，提供三种TLS策略：

```rust
use smtp_send::{send_with_config, SmtpConfig, TlsPolicy};

// 1. 机会性TLS（默认，推荐）
let config = SmtpConfig {
    tls_policy: TlsPolicy::Opportunistic,
    allow_invalid_certs: false,
    port: 25,
};

// 2. 必需TLS（更安全，但可能导致某些邮件发送失败）
let config = SmtpConfig {
    tls_policy: TlsPolicy::Required,
    allow_invalid_certs: false,
    port: 25,
};

// 3. 禁用TLS（仅用于调试，不推荐）
let config = SmtpConfig {
    tls_policy: TlsPolicy::Disabled,
    allow_invalid_certs: false,
    port: 25,
};

send_with_config(mail, config).await?;
```

### 选项3: 使用端口587 + 认证（如果你的服务器支持）

```rust
SmtpClientBuilder::new("smtp.yourserver.com", 587)
  .implicit_tls(false)  // STARTTLS
  .credentials(("username", "password"))
  .connect()
  .await?
  .send(message)
  .await?;
```

---

## 测试建议 / Testing Recommendations

### 1. 测试邮件发送能力

使用在线工具测试：
- **Mail Tester**: https://www.mail-tester.com/
- **MXToolbox**: https://mxtoolbox.com/SuperTool.aspx

### 2. 检查IP黑名单状态

```bash
# 使用MXToolbox检查
https://mxtoolbox.com/blacklists.aspx
```

### 3. 测试SMTP连接

```bash
# 手动测试SMTP连接
telnet mail.yourdomain.com 25

# 应该看到：
# 220 mail.yourdomain.com ESMTP

# 然后输入：
EHLO test.com

# 应该看到STARTTLS在支持的命令列表中：
# 250-STARTTLS
```

---

## 常见问题 / FAQ

### Q: 为什么Cloudflare拒绝我的邮件？
**A:** 因为你的IP没有PTR记录。Cloudflare（和许多其他邮件服务器）要求发送方IP必须有有效的反向DNS记录。

### Q: 如何知道是否使用了加密？
**A:** 当前代码会在日志中显示。你也可以使用Wireshark抓包查看是否有TLS握手。

### Q: 机会性TLS安全吗？
**A:** 比完全明文好，但不如强制TLS安全。它可以防止被动监听，但无法防止主动的中间人攻击（STRIPTLS攻击）。

### Q: 我应该使用哪种TLS策略？
**A:** 
- **服务器到服务器（端口25）**: 使用机会性TLS（Opportunistic）
- **客户端到服务器（端口587）**: 使用必需TLS（Required）+ 认证
- **高安全需求**: 使用必需TLS + MTA-STS

---

## 下一步行动 / Next Steps

1. ✅ **立即修复**: 配置PTR记录（联系托管商）
2. ✅ **验证邮箱**: 确保收件人邮箱地址正确
3. ✅ **配置DNS**: 添加SPF、DKIM、DMARC记录
4. ✅ **测试发送**: 使用mail-tester.com测试
5. ✅ **监控日志**: 查看新的详细错误日志

---

## 参考资源 / References

- [RFC 3207 - STARTTLS](https://tools.ietf.org/html/rfc3207)
- [RFC 5321 - SMTP](https://tools.ietf.org/html/rfc5321)
- [MTA-STS](https://tools.ietf.org/html/rfc8461)
- [mail-send文档](https://docs.rs/mail-send/)
