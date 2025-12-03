use aok::{OK, Void};
use idoh::{mx::cache::Cache, MxLookup};
use mail_send::mail_auth::common::crypto::Ed25519Key;
use mail_send::mail_auth::dkim::{Done, DkimSigner};
use mail_send::{smtp::message::Message, SmtpClientBuilder};
use mail_struct::{DomainMail, Mail};
use sk_dkim::Sk;

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

pub async fn send(mail: Mail, _retry: u64, selector: &str, sk: &Sk, sender_domain: &str) -> Void {
  // Generate DKIM signer
  let dkim_obj = sk.dkim(selector, sender_domain);
  let seed = dkim_obj.to_bytes();
  let public_key = dkim_obj.verifying_key().to_bytes();
  let ed25519_key = Ed25519Key::from_seed_and_public_key(&seed, &public_key)?;
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
