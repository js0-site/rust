use aok::{OK, Void};
use dashmap::DashMap;
use expire_cache::{Expire, map::RefVal};
use idoh::{MxLookup, mx::cache::Cache};
use mail_send::{
  SmtpClientBuilder,
  mail_auth::{
    common::crypto::Ed25519Key,
    dkim::{DkimSigner, Done},
  },
  smtp::message::Message,
};
use mail_struct::{DomainMail, Mail};
use sk_dkim::Sk;

/// 全局 DKIM signer 缓存，TTL 600 秒
/// Global DKIM signer cache with 600 seconds TTL
#[static_init::dynamic]
static DKIM_CACHE: Expire<DashMap<String, DkimSigner<Ed25519Key, Done>>> = Expire::new(600);

pub async fn send_mx<'a>(
  server: &str,
  mail: Message<'a>,
  dkim_signer: &DkimSigner<Ed25519Key, Done>,
) -> Void {
  let mut smtp = SmtpClientBuilder::new(server, 25)
    .implicit_tls(false)
    .connect()
    .await?;

  Ok(smtp.send_signed(mail, dkim_signer).await?)
}

fn dkim_signer(
  selector: &str,
  sender_domain: &str,
  sk: &Sk,
) -> RefVal<(String, DkimSigner<Ed25519Key, Done>)> {
  let cache_key = format!("{selector}.{sender_domain}");

  if let Some(signer) = DKIM_CACHE.get(&cache_key) {
    return signer;
  }

  let dkim = sk.dkim(selector, sender_domain);
  let seed = dkim.to_bytes();
  let public_key = dkim.verifying_key().to_bytes();
  let ed25519_key = Ed25519Key::from_seed_and_public_key(&seed, &public_key).unwrap();
  let dkim_signer = DkimSigner::from_key(ed25519_key)
    .domain(sender_domain)
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
    .headers(["From", "From", "Subject", "Subject", "Date", "Date", "To", "To", "Cc", "Cc"]);

  DKIM_CACHE.insert(cache_key, dkim_signer);

  dkim_signer
}

pub async fn send(mail: Mail, _retry: u64, selector: &str, sk: &Sk) -> Void {
  let sender_domain = match mail.sender.split_once("@").map(|(_, domain)| domain) {
    Some(domain) => domain,
    None => return OK,
  };

  let dkim_signer = dkim_signer(selector, sender_domain, sk);

  // let mut failed = Vec::new();
  'out: for DomainMail { domain, mail } in mail.domain_mail() {
    match Cache.mx(&domain).await {
      Ok(mx_li) => {
        if let Some(mx_li) = mx_li {
          let mut err_li = Vec::new();
          for mx in mx_li.iter() {
            match send_mx(&mx.server, mail.clone(), &dkim_signer).await {
              Ok(_) => {
                continue 'out;
              }
              Err(e) => {
                let e = e.to_string();
                log::error!("❌ {domain} → {}:25 : {e}", mx.server);
                err_li.push((mx.server.clone(), e));
              }
            }
          }
          log::error!("❌ {domain} all MX send failed");
        } else {
          log::error!("{domain} no MX dns");
        }
      }
      Err(e) => {
        log::error!("{domain} MX lookup failed: {e}");
      }
    }
    // for i in mail.rcpt_to {
    //   failed.push(i.email);
    // }
  }

  OK
}
