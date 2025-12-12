use dashmap::DashMap;
use expire_cache::{Expire, map::RefVal};
use mail_send::mail_auth::{
  common::crypto::{RsaKey, Sha256},
  dkim::{DkimSigner, Done},
};
use sk_dkim::Sk;

/// 全局 DKIM signer 缓存，TTL 600 秒
/// Global DKIM signer cache with 600 seconds TTL
#[static_init::dynamic(lazy)]
static DKIM_CACHE: Expire<DashMap<String, DkimSigner<RsaKey<Sha256>, Done>>> = Expire::new(600);

pub fn signer<'a>(
  selector: &'a str,
  sender_host: &'a str,
  sk: &'a Sk,
) -> RefVal<'a, String, DkimSigner<RsaKey<Sha256>, Done>> {
  let cache_key = format!("{selector}.{sender_host}");
  DKIM_CACHE.get_or_init(cache_key, |_| {
    let dkim = sk.dkim(selector, sender_host);
    // 将 RSA 私钥转换为 PKCS#8 DER 格式
    use rsa::pkcs8::EncodePrivateKey;
    let der = dkim.to_pkcs8_der()?;
    // 使用 PKCS#8 DER 创建 RsaKey
    let rsa_key = RsaKey::<Sha256>::from_pkcs8_der(der.as_bytes()).unwrap();
      Ok::<_, rsa::Error>(DkimSigner::from_key(rsa_key)
    .domain(sender_host)
    .selector(selector)
    /*
     * RFC 6376 安全建议：对每个头部字段列出 N+1 次（N=实际出现次数）
     * 这样可以防止攻击者在不破坏签名的情况下插入额外的恶意头部
     * 例如：From 出现 1 次，签名时列出 2 次 From，攻击者无法插入第二个 From 头部
     *
     * RFC 6376 Security Recommendation: List each header field N+1 times (N=actual occurrences)
     * This prevents attackers from injecting additional malicious headers without breaking the signature
     * Example: From appears 1 time, sign it 2 times, attackers cannot inject a 2nd From header
     */
    .headers(["From", "From", "Subject", "Subject", "Date", "Date", "To", "To", "Cc", "Cc", "Message-ID"]))
  }).expect("DKIM signer cache init failed")
}
