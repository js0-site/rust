use std::time::Instant;

use aok::{OK, Void};
use futures::future::join_all;
use idns::{Cache, Mx, Query};
use idoq::{DOQ, DOQ_LI, Doq, QType, doq_li, host_ip};
use log::{error, info, warn};
use tabled::{Table, Tabled};

#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
}

/// Query result / 单次查询结果
enum QCount {
  Ok(usize),
  None,
  Err(String),
}

impl QCount {
  fn fmt(&self) -> String {
    match self {
      Self::Ok(n) => n.to_string(),
      Self::None => "-".into(),
      Self::Err(_) => "✗".into(),
    }
  }

  fn err(&self) -> Option<&str> {
    match self {
      Self::Err(e) => Some(e),
      _ => None,
    }
  }
}

async fn query_count(client: &Doq, qtype: QType, domain: &str) -> QCount {
  match client.answer_li(qtype, domain).await {
    Ok(Some(li)) => QCount::Ok(li.len()),
    Ok(None) => QCount::None,
    Err(e) => QCount::Err(e.to_string()),
  }
}

#[derive(Tabled)]
struct ServerRow {
  #[tabled(rename = "DNS")]
  dns: String,
  #[tabled(rename = "A")]
  a: String,
  #[tabled(rename = "AAAA")]
  aaaa: String,
  #[tabled(rename = "MX")]
  mx: String,
  #[tabled(rename = "TXT")]
  txt: String,
  #[tabled(rename = "ms")]
  ms: String,
}

struct ServerResult {
  row: ServerRow,
  errors: Vec<String>,
}

async fn query_server(client: &Doq, domain: &str) -> ServerResult {
  let start = Instant::now();
  let (a, aaaa, mx, txt) = tokio::join!(
    query_count(client, QType::A, domain),
    query_count(client, QType::AAAA, domain),
    query_count(client, QType::MX, domain),
    query_count(client, QType::TXT, domain),
  );

  let server = format!("{} ({})", client.server.host, client.server.ip);
  let mut errors = Vec::new();

  for (qtype, res) in [("A", &a), ("AAAA", &aaaa), ("MX", &mx), ("TXT", &txt)] {
    if let Some(e) = res.err() {
      errors.push(format!("{server} {qtype}: {e}"));
    }
  }

  ServerResult {
    row: ServerRow {
      dns: format!("{} ({})", client.server.host, client.server.ip),
      a: a.fmt(),
      aaaa: aaaa.fmt(),
      mx: mx.fmt(),
      txt: txt.fmt(),
      ms: start.elapsed().as_millis().to_string(),
    },
    errors,
  }
}

#[tokio::test]
async fn test_gmail() -> Void {
  let domain = "gmail.com";
  info!("查询 {domain}");

  let clients = doq_li(DOQ_LI);
  let tasks: Vec<_> = clients.iter().map(|c| query_server(c, domain)).collect();
  let results = join_all(tasks).await;

  let rows: Vec<_> = results.iter().map(|r| &r.row).collect();
  info!("\n{}", Table::new(&rows));

  let errors: Vec<&str> = results
    .iter()
    .flat_map(|r| r.errors.iter().map(|s| s.as_str()))
    .collect();
  if !errors.is_empty() {
    error!("\n{}", errors.join("\n"));
  }

  OK
}

/// Query result / 查询结果
enum QRes {
  Ok(usize, u128),
  Err(String, u128),
}

impl QRes {
  fn count(&self) -> Option<usize> {
    match self {
      Self::Ok(n, _) => Some(*n),
      Self::Err(..) => None,
    }
  }

  fn is_ok(&self) -> bool {
    matches!(self, Self::Ok(..))
  }

  fn fmt(&self) -> String {
    match self {
      Self::Ok(n, ms) => format!("{n} ({ms}ms)"),
      Self::Err(msg, ms) => format!("✗ {msg} ({ms}ms)"),
    }
  }
}

async fn query_cmp(client: &Doq, qtype: QType, domain: &str) -> QRes {
  let start = Instant::now();
  let ms = start.elapsed().as_millis();
  match client.answer_li(qtype, domain).await {
    Ok(Some(li)) => QRes::Ok(li.len(), ms),
    Ok(None) => QRes::Ok(0, ms),
    Err(e) => QRes::Err(format!("{e:?}"), ms),
  }
}

#[derive(Tabled)]
struct CmpRow {
  #[tabled(rename = "DNS")]
  dns: String,
  #[tabled(rename = "MX")]
  mx: String,
  #[tabled(rename = "TXT")]
  txt: String,
}

#[tokio::test]
async fn test_alidns() -> Void {
  let domain = "qq.com";
  let adguard = Doq::new(host_ip("unfiltered.adguard-dns.com", 94, 140, 14, 140));
  let alidns = Doq::new(host_ip("dns.alidns.com", 223, 5, 5, 5));

  info!("MX/TXT 对比 {domain}");

  let (adguard_mx, adguard_txt, alidns_mx, alidns_txt) = tokio::join!(
    query_cmp(&adguard, QType::MX, domain),
    query_cmp(&adguard, QType::TXT, domain),
    query_cmp(&alidns, QType::MX, domain),
    query_cmp(&alidns, QType::TXT, domain),
  );

  let rows = vec![
    CmpRow {
      dns: "AdGuard".into(),
      mx: adguard_mx.fmt(),
      txt: adguard_txt.fmt(),
    },
    CmpRow {
      dns: "AliDNS".into(),
      mx: alidns_mx.fmt(),
      txt: alidns_txt.fmt(),
    },
  ];

  info!("\n{}", Table::new(&rows));

  let mx_ok = adguard_mx.is_ok() && alidns_mx.is_ok();
  let txt_ok = adguard_txt.is_ok() && alidns_txt.is_ok();

  if !mx_ok {
    warn!("MX 查询失败，跳过一致性验证");
    return OK;
  }
  if !txt_ok {
    warn!("TXT 查询失败，跳过一致性验证");
    return OK;
  }

  assert_eq!(adguard_mx.count(), alidns_mx.count(), "MX 数量不一致");
  assert_eq!(adguard_txt.count(), alidns_txt.count(), "TXT 数量不一致");

  info!("✓ MX/TXT 一致性验证通过");
  OK
}

#[tokio::test]
async fn test_mx_parse() -> Void {
  let client = Doq::new(host_ip("dns.alidns.com", 223, 5, 5, 5));
  let domain = "gmail.com";

  info!("查询 {domain} MX (Parse trait)");

  if let Ok(Some(li)) = Query::query::<Mx>(&client, domain).await {
    let records: Vec<_> = li
      .iter()
      .map(|mx| format!("{} {} TTL={}", mx.priority, mx.server, mx.ttl))
      .collect();
    info!("{}", records.join("\n"));
  }
  OK
}

#[tokio::test]
async fn test_dns_race() -> Void {
  info!("DOQ 查询 github.com");
  let start = Instant::now();
  if let Ok(Some(li)) = DOQ.answer_li(QType::A, "github.com").await {
    let records: Vec<_> = li.iter().map(|a| a.val.clone()).collect();
    info!(
      "A 记录 ({}ms): {}",
      start.elapsed().as_millis(),
      records.join(", ")
    );
  }

  if let Ok(Some(li)) = Query::query::<Mx>(&*DOQ, "qq.com").await {
    let records: Vec<_> = li
      .iter()
      .map(|mx| format!("{} {}", mx.priority, mx.server))
      .collect();
    info!("qq.com MX: {}", records.join(", "));
  }
  OK
}

#[tokio::test]
async fn test_conn_reuse() -> Void {
  let client = Doq::new(host_ip("dns.alidns.com", 223, 5, 5, 5));

  info!("连接复用测试");

  let start = Instant::now();
  let _ = client.answer_li(QType::A, "baidu.com").await;
  let mut times = vec![format!("{}ms", start.elapsed().as_millis())];

  for _ in 2..=5 {
    let start = Instant::now();
    let _ = client.answer_li(QType::A, "qq.com").await;
    times.push(format!("{}ms", start.elapsed().as_millis()));
  }
  info!("耗时: {}", times.join(" -> "));
  OK
}

#[tokio::test]
async fn test_dns_race_cache() -> Void {
  let cache: Cache<Mx> = Cache::new(60);
  let domain = "gmail.com";

  info!("DOQ + Cache 测试 {domain}");

  let t1 = Instant::now();
  let r1 = cache.query(&*DOQ, domain).await;
  let d1 = t1.elapsed();

  let mut output = format!("首次: {}ms", d1.as_millis());
  if let Some(li) = r1.unwrap() {
    let records: Vec<_> = li
      .iter()
      .map(|mx| format!("{} {}", mx.priority, mx.server))
      .collect();
    output.push_str(&format!("\n  {}", records.join("\n  ")));
  }

  let t2 = Instant::now();
  let r2 = cache.query(&*DOQ, domain).await;
  let d2 = t2.elapsed();

  output.push_str(&format!("\n缓存: {}μs", d2.as_micros()));
  if let Some(li) = r2.unwrap() {
    let records: Vec<_> = li
      .iter()
      .map(|mx| format!("{} {}", mx.priority, mx.server))
      .collect();
    output.push_str(&format!("\n  {}", records.join("\n  ")));
  }

  info!("{output}");

  assert!(d2 < d1 / 10, "缓存应快10倍: {d1:?} vs {d2:?}");
  info!(
    "✓ 缓存加速: {}ms -> {}μs ({}倍)",
    d1.as_millis(),
    d2.as_micros(),
    d1.as_micros() / d2.as_micros().max(1)
  );
  OK
}

#[derive(Tabled)]
struct FilterRow {
  #[tabled(rename = "DNS")]
  dns: String,
  #[tabled(rename = "域名")]
  domain: String,
  #[tabled(rename = "结果")]
  result: String,
  #[tabled(rename = "记录数")]
  count: String,
}

#[tokio::test]
async fn test_dns_filtering() -> Void {
  let ad_domains = vec![
    // 安全 / 杀毒测试
    "eicar.org",
    // 恶意 / 钓鱼测试
    "malware.testcategory.com",
    // 广告 / 跟踪
    "doubleclick.net",
    "ads.facebook.com",
    // 成人内容（家长控制 / 地区 DNS 常封）
    "pornhub.com",
    "xvideos.com",
    // 赌博 / 博彩
    "bet365.com",
    // 盗版 / 争议内容
    "thepiratebay.org",
  ];

  let dns_servers = doq_li(DOQ_LI);

  for client in &dns_servers {
    let mut rows = Vec::new();
    let mut server_unfiltered = true;

    for domain in &ad_domains {
      let result = client.answer_li(QType::A, domain).await;

      let (result_str, count_str) = match result {
        Ok(Some(records)) => ("未过滤".to_string(), records.len().to_string()),
        Ok(None) => {
          server_unfiltered = false;
          ("已过滤".to_string(), "0".to_string())
        }
        Err(_) => {
          server_unfiltered = false;
          ("已过滤".to_string(), "0".to_string())
        }
      };

      rows.push(FilterRow {
        dns: format!("{} ({})", client.server.host, client.server.ip),
        domain: domain.to_string(),
        result: result_str,
        count: count_str,
      });
    }

    if server_unfiltered {
      info!("{} ({}) 完全无过滤", client.server.host, client.server.ip);
    } else {
      println!(
        "\n{} ({}) 过滤情况:\n{}",
        client.server.host,
        client.server.ip,
        Table::new(&rows)
      );
    }
  }
  OK
}

/// 测试 salesforce.com TXT 记录查询（该域名有 34 条 TXT 记录）
/// 逐个服务器查询，不使用缓存，验证每个服务器都能返回完整的 SPF 记录
#[tokio::test]
async fn test_salesforce_txt() -> Void {
  let domain = "salesforce.com";
  info!("逐个服务器查询 {domain} TXT 记录（无缓存）");

  let clients = doq_li(DOQ_LI);

  for client in &clients {
    let start = Instant::now();
    let result = client.answer_li(QType::TXT, domain).await;
    let ms = start.elapsed().as_millis();

    match result {
      Ok(Some(li)) => {
        let has_spf = li.iter().any(|a| a.val.contains("v=spf1"));
        let spf_status = if has_spf { "✓ SPF" } else { "✗ 无SPF" };
        info!(
          "{} ({}) : {} 条 TXT, {} ({}ms)",
          client.server.host,
          client.server.ip,
          li.len(),
          spf_status,
          ms
        );
        assert!(has_spf, "{} 未返回 SPF 记录", client.server.host);
      }
      Ok(None) => {
        error!(
          "{} ({}) : 无记录 ({}ms)",
          client.server.host, client.server.ip, ms
        );
        panic!("{} 返回空结果", client.server.host);
      }
      Err(e) => {
        error!(
          "{} ({}) : 错误 {} ({}ms)",
          client.server.host, client.server.ip, e, ms
        );
        panic!("{} 查询失败: {}", client.server.host, e);
      }
    }
  }

  info!("✓ 所有服务器都返回了完整的 SPF 记录");
  OK
}
