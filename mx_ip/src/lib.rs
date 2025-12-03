#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod error;
pub use error::{Error, Result};

use std::{
  collections::HashSet,
  net::{Ipv4Addr, Ipv6Addr},
  sync::Arc,
  time::Duration,
};

use futures::future::join_all;
use rand::seq::SliceRandom;
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Default)]
pub struct MxIp {
  pub v6_li: Vec<Ipv6Addr>,
  pub v4_li: Vec<Ipv4Addr>,
}

#[derive(Deserialize, Debug)]
struct DnsAnswer {
  #[serde(rename = "type")]
  record_type: u16,
  data: String,
}

#[derive(Deserialize, Debug)]
struct DnsResponse {
  #[serde(rename = "Status")]
  status: i32,
  #[serde(rename = "Answer")]
  answer: Option<Vec<DnsAnswer>>,
}

const PROVIDERS: &[&str] = &[
  "https://dns.alidns.com/resolve",
  "https://cloudflare-dns.com/dns-query",
  "https://dns.google/resolve",
  "https://doh.pub/dns-query", // Tencent
];

async fn query_doh(client: &Client, name: &str, record_type: &str) -> Result<Vec<String>> {
  let mut providers = PROVIDERS.to_vec();
  providers.shuffle(&mut rand::rng());

  for provider in providers {
    let url = format!("{}?name={}&type={}", provider, name, record_type);
    let resp = client
      .get(&url)
      .header("accept", "application/dns-json")
      .timeout(Duration::from_secs(5))
      .send()
      .await;

    match resp {
      Ok(resp) => {
        if let Ok(dns_resp) = resp.json::<DnsResponse>().await {
          if dns_resp.status == 0 {
            if let Some(answers) = dns_resp.answer {
              let results: Vec<String> = answers
                .into_iter()
                .filter(|a| {
                  // MX = 15, A = 1, AAAA = 28
                  match record_type {
                    "MX" => a.record_type == 15,
                    "A" => a.record_type == 1,
                    "AAAA" => a.record_type == 28,
                    _ => false,
                  }
                })
                .map(|a| a.data)
                .collect();
              return Ok(results);
            } else {
              // No answer section, but status 0 means no records found (NODATA)
              return Ok(vec![]);
            }
          }
        }
      }
      Err(_) => continue, // Try next provider
    }
  }

  Err(Error::Dns(format!(
    "Failed to resolve {} type {} from all providers",
    name, record_type
  )))
}

pub async fn mx_ip(host: impl AsRef<str>) -> Result<MxIp> {
  let host = host.as_ref();
  let client = Client::builder()
    .timeout(Duration::from_secs(10))
    .build()?;

  // 1. Get MX records
  let mx_records = query_doh(&client, host, "MX").await?;
  if mx_records.is_empty() {
    return Err(Error::NoMxRecords(host.to_string()));
  }

  // Parse MX records to get hostnames. Format: "priority hostname"
  let mut mx_hosts = HashSet::new();
  for record in mx_records {
    let parts: Vec<&str> = record.split_whitespace().collect();
    if parts.len() >= 2 {
      // The hostname might have a trailing dot
      let hostname = parts[1].trim_end_matches('.');
      mx_hosts.insert(hostname.to_string());
    }
  }

  if mx_hosts.is_empty() {
    return Err(Error::NoMxRecords(host.to_string()));
  }

  // 2. Resolve IPs for each MX host
  let mut v4_li = HashSet::new();
  let mut v6_li = HashSet::new();

  let mut futures = Vec::new();
  for mx_host in mx_hosts {
    let client_clone = client.clone();
    let mx_host_clone = mx_host.clone();
    futures.push(tokio::spawn(async move {
      let v4 = query_doh(&client_clone, &mx_host_clone, "A").await;
      let v6 = query_doh(&client_clone, &mx_host_clone, "AAAA").await;
      (v4, v6)
    }));
  }

  let results = join_all(futures).await;

  for res in results {
    if let Ok((v4_res, v6_res)) = res {
      if let Ok(ips) = v4_res {
        for ip in ips {
          if let Ok(addr) = ip.parse::<Ipv4Addr>() {
            v4_li.insert(addr);
          }
        }
      }
      if let Ok(ips) = v6_res {
        for ip in ips {
          if let Ok(addr) = ip.parse::<Ipv6Addr>() {
            v6_li.insert(addr);
          }
        }
      }
    }
  }

  if v4_li.is_empty() && v6_li.is_empty() {
    return Err(Error::NoIpRecords(host.to_string()));
  }

  Ok(MxIp {
    v4_li: v4_li.into_iter().collect(),
    v6_li: v6_li.into_iter().collect(),
  })
}
