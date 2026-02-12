use std::{
  net::{IpAddr, Ipv4Addr, Ipv6Addr},
  time::Instant,
};

use aok::{OK, Void};
use idns::{DnsRace, Query, Spf, spf::Status};
use idot::{DOT_LI, dot_li};

mod site {
  pub const GMAIL: &str = "gmail.com";
  pub const OUTLOOK: &str = "outlook.com";
  pub const QQ: &str = "qq.com";
  pub const MAIL163: &str = "163.com";
  pub const YAHOO: &str = "yahoo.com";
  pub const ICLOUD: &str = "icloud.com";
  pub const SALESFORCE: &str = "salesforce.com";
  pub const INVALID: &str = "example.invalid";
}

#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
}

#[tokio::test]
async fn test_spf() -> Void {
  use site::*;

  let dns = DnsRace::new(dot_li(DOT_LI));
  let domains = [GMAIL, OUTLOOK, QQ, MAIL163, YAHOO, ICLOUD];

  for domain in domains {
    log::info!("# {domain}");
    if let Some(li) = dns.query::<Spf>(domain).await? {
      for spf in li {
        log::info!("{spf:?}");
      }
    } else {
      log::info!("no SPF record");
    }
  }

  OK
}

#[tokio::test]
async fn test_spf_cache() -> Void {
  let dns = DnsRace::new(dot_li(DOT_LI));

  let domain = "icloud.com";
  let ip: IpAddr = "17.41.0.1".parse()?;

  let t1 = Instant::now();
  let r1 = Spf::verify(&dns, domain, ip).await;
  let d1 = t1.elapsed();

  let t2 = Instant::now();
  let r2 = Spf::verify(&dns, domain, ip).await;
  let d2 = t2.elapsed();

  log::info!(
    "first: {d1:?}, second: {d2:?}, speedup: {:.1}x",
    d1.as_secs_f64() / d2.as_secs_f64()
  );
  log::info!("result: {r1:?} / {r2:?}");

  assert!(d2 < d1 / 10, "cache not effective: {d1:?} vs {d2:?}");

  OK
}

/// 测试 salesforce.com TXT 记录查询（该域名有 34 条 TXT 记录）
#[tokio::test]
async fn test_salesforce_txt() -> Void {
  use idns::QType;

  let dns = DnsRace::new(dot_li(DOT_LI));
  let answers = dns.answer_li(QType::TXT, "salesforce.com").await?;

  let count = answers.as_ref().map(|v| v.len()).unwrap_or(0);
  let has_spf = answers
    .as_ref()
    .map(|li| li.iter().any(|a| a.val.contains("v=spf1")))
    .unwrap_or(false);

  log::info!("salesforce.com: {} TXT records, SPF={}", count, has_spf);
  assert!(has_spf, "SPF record not found in {} TXT records", count);

  OK
}

/// IPv6 SPF 验证测试 / IPv6 SPF verification test
#[tokio::test]
async fn test_spf_ipv6() -> Void {
  let dns = DnsRace::new(dot_li(DOT_LI));

  // icloud.com 有 IPv6 SPF 规则
  let domain = "icloud.com";
  let spf = dns
    .query::<Spf>(domain)
    .await?
    .and_then(|li| li.into_iter().next())
    .expect("icloud.com should have SPF");

  // 从 ip6 范围取一个 IP 测试 Pass
  if let Some((r, _)) = spf.ip6.first() {
    let ip = IpAddr::V6(Ipv6Addr::from(r.start + 1));
    let result = Spf::verify(&dns, domain, ip).await;
    log::info!("✓ {domain} + {ip} = {result:?}");
    assert_eq!(result, Status::Pass, "IPv6 in range should Pass");
  } else {
    panic!("icloud.com should have IPv6 SPF rules");
  }

  // 私有 IPv6 应该 SoftFail
  let private_v6: IpAddr = "fd00::1".parse()?;
  let result = Spf::verify(&dns, domain, private_v6).await;
  log::info!("✓ {domain} + {private_v6} = {result:?}");
  assert!(
    matches!(result, Status::Fail | Status::SoftFail),
    "private IPv6 should Fail/SoftFail"
  );

  OK
}

/// 动态 SPF 验证测试：根据查询到的 SPF 记录生成测试 IP
#[tokio::test]
async fn test_verify_dynamic() -> Void {
  use site::*;

  let dns = DnsRace::new(dot_li(DOT_LI));
  let domains = [GMAIL, OUTLOOK, QQ, ICLOUD, SALESFORCE];

  // 内网 IP，肯定不在任何 SPF 记录中
  let private_ip: IpAddr = "192.168.1.1".parse()?;

  for domain in domains {
    log::info!("=== {domain} ===");

    // 查询 SPF 记录
    let spf_li = match dns.query::<Spf>(domain).await? {
      Some(li) if !li.is_empty() => li,
      _ => {
        log::warn!("{domain}: no SPF record");
        continue;
      }
    };

    let spf = &spf_li[0];

    // 从范围中取一个 IP 测试 Pass / Get an IP from range for Pass test
    let pass_ip: Option<IpAddr> = if let Some((r, _)) = spf.ip4.first() {
      Some(IpAddr::V4(Ipv4Addr::from(r.start + 1)))
    } else {
      spf
        .ip6
        .first()
        .map(|(r, _)| IpAddr::V6(Ipv6Addr::from(r.start + 1)))
    };

    if let Some(ip) = pass_ip {
      let result = Spf::verify(&dns, domain, ip).await;
      log::info!(
        "{} {domain} + {ip} = {result:?} (expected Pass)",
        if result == Status::Pass { "✓" } else { "✗" }
      );
      assert_eq!(result, Status::Pass, "{domain} + {ip} should Pass");
    } else {
      log::info!("{domain}: no direct IP rules, skip Pass test");
    }

    // 内网 IP 测试 Fail/SoftFail
    let result = Spf::verify(&dns, domain, private_ip).await;
    let ok = matches!(result, Status::Fail | Status::SoftFail);
    log::info!(
      "{} {domain} + {private_ip} = {result:?} (expected Fail/SoftFail)",
      if ok { "✓" } else { "✗" }
    );
    assert!(ok, "{domain} + {private_ip} should Fail/SoftFail");
  }

  // 测试无 SPF 记录的域名
  let result = Spf::verify(&dns, INVALID, private_ip).await;
  log::info!("✓ {INVALID} = {result:?} (expected None)");
  assert_eq!(result, Status::None);

  OK
}
