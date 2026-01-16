#![cfg_attr(docsrs, feature(doc_cfg))]

mod error;
pub use error::{Error, Result};
use idns::{Answer, QType};
use ireq::REQ;
use serde::Deserialize;
use serde_json::{Value, from_slice, from_value};

/// DoH 服务器列表
pub static DOH_LI: &[&str] = &[
  "doh.pub/resolve",              // 腾讯
  "dns.google/resolve",           // Google
  "cloudflare-dns.com/dns-query", // Cloudflare
  "doh.sb/dns-query",             // DNS.SB
  "doh.360.cn/resolve",           // 360
  "dns.nextdns.io",               // NextDNS
  "dns.alidns.com/resolve",       // 阿里
];

#[derive(Debug, Deserialize)]
struct DnsAnswer {
  name: String,
  #[serde(rename = "type")]
  type_id: u16,
  #[serde(rename = "TTL")]
  ttl: u32,
  data: String,
}

impl From<DnsAnswer> for Answer {
  fn from(a: DnsAnswer) -> Self {
    Answer {
      name: a.name,
      val: a.data,
      type_id: a.type_id,
      ttl: a.ttl,
    }
  }
}

/// DoH 客户端
#[derive(Debug, Clone)]
pub struct Doh {
  pub url: String,
}

impl Doh {
  pub fn new(url: impl Into<String>) -> Self {
    Self { url: url.into() }
  }

  /// 查询 DNS 记录
  pub async fn query(&self, name: &str, qtype: QType) -> Result<Option<Vec<Answer>>> {
    let url = format!("https://{}?name={}&type={}", self.url, name, qtype as u16);

    let req = REQ.get(&url).header("Accept", "application/dns-json");
    let res = ireq::req(req).await?;

    let json: Value = from_slice(&res)?;

    if let Some(status) = json.get("Status").and_then(|v| v.as_u64())
      && status != 0
    {
      return Ok(None);
    }

    if let Some(answer_li) = json.get("Answer") {
      let mut li: Vec<DnsAnswer> = from_value(answer_li.clone())?;

      // TXT 记录去除引号
      li.iter_mut().for_each(|i| {
        if i.type_id == 16 && i.data.starts_with('"') && i.data.ends_with('"') {
          i.data = i.data[1..i.data.len() - 1].into();
        }
      });

      return Ok(Some(li.into_iter().map(Into::into).collect()));
    }

    Ok(None)
  }
}

impl idns::Query for Doh {
  type Error = Error;

  async fn answer_li(&self, qtype: QType, name: &str) -> Result<Option<Vec<Answer>>> {
    self.query(name, qtype).await
  }
}

/// 从 DoH URL 列表创建 Doh 客户端
pub fn doh_li(li: &[&str]) -> Vec<Doh> {
  li.iter().map(|s| Doh::new(*s)).collect()
}

#[cfg(feature = "static")]
#[static_init::dynamic(lazy)]
pub static DOH: idns::DnsRace<Doh> = idns::DnsRace::new(doh_li(DOH_LI));
