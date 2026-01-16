use std::time::Instant;

use aok::{OK, Void};
use futures::future::join_all;
use idns::{Mx, Query};
use idoh::{DOH, DOH_LI, Doh, doh_li};
use log::{error, info};
use tabled::{Table, Tabled};

#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
}

/// 单次查询结果
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

async fn query_count(client: &Doh, qtype: idns::QType, domain: &str) -> QCount {
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

async fn query_server(client: &Doh, domain: &str) -> ServerResult {
  let start = Instant::now();
  let (a, aaaa, mx, txt) = tokio::join!(
    query_count(client, idns::QType::A, domain),
    query_count(client, idns::QType::AAAA, domain),
    query_count(client, idns::QType::MX, domain),
    query_count(client, idns::QType::TXT, domain),
  );

  let mut errors = Vec::new();

  for (qtype, res) in [("A", &a), ("AAAA", &aaaa), ("MX", &mx), ("TXT", &txt)] {
    if let Some(e) = res.err() {
      errors.push(format!("{} {qtype}: {e}", client.url));
    }
  }

  ServerResult {
    row: ServerRow {
      dns: client.url.clone(),
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

  let clients = doh_li(DOH_LI);
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

#[tokio::test]
async fn test_alidns() -> Void {
  let client = Doh::new("dns.alidns.com/resolve");
  let domain = "qq.com";

  info!("查询 {domain} A 记录");

  if let Ok(Some(li)) = client.answer_li(idns::QType::A, domain).await {
    let records: Vec<_> = li.iter().map(|a| a.val.clone()).collect();
    info!("A: {}", records.join(", "));
  }
  OK
}

#[tokio::test]
async fn test_mx_parse() -> Void {
  let client = Doh::new("dns.alidns.com/resolve");
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
  info!("DOH 查询 github.com");
  let start = Instant::now();
  if let Ok(Some(li)) = DOH.answer_li(idns::QType::A, "github.com").await {
    let records: Vec<_> = li.iter().map(|a| a.val.clone()).collect();
    info!(
      "A 记录 ({}ms): {}",
      start.elapsed().as_millis(),
      records.join(", ")
    );
  }

  if let Ok(Some(li)) = Query::query::<Mx>(&*DOH, "qq.com").await {
    let records: Vec<_> = li
      .iter()
      .map(|mx| format!("{} {}", mx.priority, mx.server))
      .collect();
    info!("qq.com MX: {}", records.join(", "));
  }
  OK
}
