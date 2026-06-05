use std::time::Instant;

use aok::{OK, Void};
use idns::Query;
use idot::{DOT_LI, Dot, QType, dot_li, host_ip};
use log::info;

#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
}

#[tokio::test]
async fn test_query() -> Void {
  let client = Dot::new(host_ip("cloudflare-dns.com", 1, 1, 1, 1));

  if let Some(answers) = client.answer_li(QType::A, "google.com").await? {
    for a in &answers {
      info!("{} {} TTL={}", a.name, a.val, a.ttl);
    }
    assert!(!answers.is_empty());
  }
  OK
}

#[tokio::test]
async fn test_aaaa() -> Void {
  let client = Dot::new(host_ip("dns.google", 8, 8, 8, 8));

  if let Some(answers) = client.answer_li(QType::AAAA, "google.com").await? {
    for a in &answers {
      info!("AAAA: {} {}", a.name, a.val);
    }
  }
  OK
}

#[tokio::test]
async fn test_race() -> Void {
  let race = idns::DnsRace::new(dot_li(DOT_LI));

  if let Some(answers) = race.answer_li(QType::A, "github.com").await? {
    for a in &answers {
      info!("race: {} {}", a.name, a.val);
    }
    assert!(!answers.is_empty());
  }
  OK
}

/// 测试 salesforce.com MX 记录查询
#[tokio::test]
async fn test_salesforce_mx() -> Void {
  let race = idns::DnsRace::new(dot_li(DOT_LI));
  let answers = race.answer_li(QType::MX, "salesforce.com").await?;

  let count = answers.as_ref().map(|v| v.len()).unwrap_or(0);
  info!("salesforce.com: {} MX records", count);

  assert!(count >= 2, "expected at least 2 MX records, got {}", count);

  if let Some(li) = &answers {
    for a in li {
      info!("  MX: {} {}", a.val, a.name);
    }
  }

  OK
}

/// 测试所有 DoT 服务器返回的 salesforce.com TXT 记录数量一致性
#[tokio::test]
async fn test_salesforce_txt_all_servers() -> Void {
  let mut results = Vec::new();

  for server in DOT_LI {
    let client = Dot::new(server.clone());
    let start = Instant::now();

    match client.answer_li(QType::TXT, "salesforce.com").await {
      Ok(answers) => {
        let count = answers.as_ref().map(|v| v.len()).unwrap_or(0);
        let has_spf = answers
          .as_ref()
          .map(|li| li.iter().any(|a| a.val.contains("v=spf1")))
          .unwrap_or(false);
        let ms = start.elapsed().as_millis();

        info!(
          "{} ({}) : {} 条 TXT, SPF={} ({}ms)",
          server.host, server.ip, count, has_spf, ms
        );
        results.push((server.host.clone(), count, has_spf));
      }
      Err(e) => {
        info!("{} ({}) : 错误 {}", server.host, server.ip, e);
      }
    }
  }

  // 验证所有服务器都返回了 SPF 记录
  for (host, count, has_spf) in &results {
    assert!(*has_spf, "{} 返回 {} 条 TXT 但无 SPF 记录", host, count);
  }

  // 验证记录数量一致性（允许小幅差异）
  if results.len() >= 2 {
    let counts: Vec<_> = results.iter().map(|(_, c, _)| *c).collect();
    let min = *counts.iter().min().unwrap();
    let max = *counts.iter().max().unwrap();
    info!("TXT 记录数量范围: {} - {}", min, max);
    assert!(
      max - min <= 5,
      "服务器返回的 TXT 记录数量差异过大: {} vs {}",
      min,
      max
    );
  }

  OK
}
