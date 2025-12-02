# SMTP发送故障排查指南 / SMTP Sending Troubleshooting Guide

## 当前遇到的错误分析 / Current Error Analysis

### 1. Google邮箱错误 (550 5.1.1)

**错误信息 / Error Message:**
```
Code: 550, Enhanced code: 5.1.1
Message: The email account that you tried to reach does not exist.
```

**原因 / Cause:**
- 收件人邮箱地址不存在
- 邮箱地址拼写错误
- 邮箱已被删除或禁用

**解决方案 / Solution:**
1. 仔细检查收件人邮箱地址的拼写
2. 确认收件人邮箱确实存在
3. 在发送前验证邮箱地址的有效性

---

### 2. Cloudflare反向DNS错误 (550 0.0.0)

**错误信息 / Error Message:**
```
Code: 550, Enhanced code: 0.0.0
Message: Sender IP reverse lookup rejected (2605:a141:2288:4651::)
```

**原因 / Cause:**
发送服务器的IP地址 `2605:a141:2288:4651::` 没有正确配置反向DNS（PTR记录）。

**这是什么？/ What is this?**
反向DNS（PTR记录）是将IP地址映射回域名的DNS记录。许多邮件服务器要求发送方IP必须有有效的PTR记录，以防止垃圾邮件。

**解决方案 / Solution:**

#### 步骤1: 检查当前PTR记录
```bash
# 对于IPv6地址
dig -x 2605:a141:2288:4651::

# 对于IPv4地址（如果有）
dig -x YOUR_IPV4_ADDRESS
```

#### 步骤2: 配置PTR记录
你需要联系你的服务器托管商（ISP或云服务提供商）来配置PTR记录。

**示例配置：**
- **IP地址**: `2605:a141:2288:4651::`
- **PTR记录应指向**: `mail.yourdomain.com`
- **确保**: `mail.yourdomain.com` 的A/AAAA记录也指向该IP

**常见云服务商配置方法：**

- **AWS**: EC2 → Request to Remove Email Sending Limitations
- **Google Cloud**: 在Cloud Console中设置PTR记录
- **DigitalOcean**: Networking → Domains → Add PTR record
- **Vultr**: Settings → IPv4/IPv6 → Reverse DNS

#### 步骤3: 验证配置
```bash
# 检查PTR记录
dig -x 2605:a141:2288:4651::

# 检查正向解析
dig mail.yourdomain.com AAAA

# 两者应该匹配
```

---

## 加密通讯改进 / Encryption Improvements

### 当前状态 / Current Status
- ❌ 使用端口25（明文）
- ❌ `implicit_tls(false)` - 禁用隐式TLS
- ⚠️ 没有STARTTLS支持

### 已实施的改进 / Implemented Improvements
代码已更新为：
- ✅ 支持STARTTLS加密（在端口25上）
- ✅ 允许自签名证书（可配置）
- ✅ 更详细的错误日志
- ✅ 双语错误提示

### 进一步的安全建议 / Further Security Recommendations

#### 选项1: 使用端口587 + STARTTLS（推荐用于客户端到服务器）
```rust
SmtpClientBuilder::new(&mx.server, 587)
  .implicit_tls(false)
  .credentials(("username", "password"))  // 如果需要认证
  .connect()
  .await
```

#### 选项2: 使用端口465 + 隐式TLS
```rust
SmtpClientBuilder::new(&mx.server, 465)
  .implicit_tls(true)
  .connect()
  .await
```

#### 选项3: 端口25 + STARTTLS（当前实现，用于服务器到服务器）
```rust
SmtpClientBuilder::new(&mx.server, 25)
  .implicit_tls(false)
  .allow_invalid_certs()  // 仅在测试环境使用
  .connect()
  .await
```

---

## 完整的邮件服务器配置检查清单 / Complete Mail Server Configuration Checklist

### DNS配置 / DNS Configuration
- [ ] **A记录**: `mail.yourdomain.com` → 你的IPv4地址
- [ ] **AAAA记录**: `mail.yourdomain.com` → 你的IPv6地址
- [ ] **MX记录**: `yourdomain.com` → `mail.yourdomain.com`
- [ ] **PTR记录**: 你的IP → `mail.yourdomain.com`
- [ ] **SPF记录**: `v=spf1 ip4:YOUR_IPV4 ip6:YOUR_IPV6 -all`
- [ ] **DKIM记录**: 配置DKIM签名
- [ ] **DMARC记录**: `v=DMARC1; p=quarantine; rua=mailto:postmaster@yourdomain.com`

### 网络配置 / Network Configuration
- [ ] 确保端口25出站未被阻止
- [ ] 检查防火墙规则
- [ ] 确认IP不在黑名单中

### 测试工具 / Testing Tools
```bash
# 检查MX记录
dig yourdomain.com MX

# 检查SPF记录
dig yourdomain.com TXT

# 检查PTR记录
dig -x YOUR_IP

# 测试SMTP连接
telnet mail.yourdomain.com 25

# 检查IP黑名单状态
# 访问: https://mxtoolbox.com/blacklists.aspx
```

---

## 代码使用示例 / Code Usage Example

```rust
use mail_struct::Mail;
use smtp_send::send;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    log_init::init();

    // 创建邮件
    let mail = Mail {
        from: "sender@yourdomain.com".to_string(),
        to_li: vec![
            "recipient@example.com".to_string(),
        ],
        subject: "Test Email".to_string(),
        body: "This is a test email.".to_string(),
        // ... 其他字段
    };

    // 发送邮件
    send(mail).await?;

    Ok(())
}
```

---

## 常见问题 / FAQ

### Q1: 为什么使用端口25而不是587或465？
**A:** 端口25用于服务器到服务器（MTA到MTA）的邮件传输。端口587用于客户端到服务器（MUA到MTA）的提交，通常需要认证。

### Q2: 什么是STARTTLS？
**A:** STARTTLS是一种在明文连接上升级到加密连接的协议。它允许在端口25上进行加密通讯。

### Q3: 为什么需要PTR记录？
**A:** PTR记录用于反向DNS查找，许多邮件服务器用它来验证发送方的身份，防止垃圾邮件。

### Q4: 如何测试邮件发送？
**A:** 可以使用 `mail-tester.com` 或 `mxtoolbox.com` 等在线工具测试邮件配置和发送能力。

---

## 参考资源 / References

- [RFC 5321 - SMTP](https://tools.ietf.org/html/rfc5321)
- [RFC 3207 - STARTTLS](https://tools.ietf.org/html/rfc3207)
- [SPF Record Syntax](http://www.open-spf.org/SPF_Record_Syntax/)
- [DKIM Core](https://tools.ietf.org/html/rfc6376)
- [DMARC](https://dmarc.org/)
