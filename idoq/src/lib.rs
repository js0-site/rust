#![cfg_attr(docsrs, feature(doc_cfg))]

mod doq;
mod error;
mod parser;

use std::net::{IpAddr, Ipv4Addr};

pub use doq::Doq;
pub use error::{Error, Result};
use idns::Answer;
pub use idns::QType;
use hipstr::HipStr;

/// Host with IP / 主机与 IP
#[derive(Debug, Clone)]
pub struct HostIp {
  pub host: HipStr<'static>,
  pub ip: IpAddr,
}

impl idns::Query for Doq {
  type Error = Error;

  async fn answer_li(&self, qtype: QType, name: &str) -> Result<Option<Vec<Answer>>> {
    self.query(name, qtype).await
  }
}

/// Create HostIp from host and IPv4 / 从主机名和 IPv4 创建 HostIp
pub const fn host_ip(host: &'static str, a: u8, b: u8, c: u8, d: u8) -> HostIp {
  HostIp {
    host: HipStr::borrowed(host),
    ip: IpAddr::V4(Ipv4Addr::new(a, b, c, d)),
  }
}

/// DoQ server hostnames / DoQ 服务器域名
pub mod dns {
  pub const ALIDNS: &str = "dns.alidns.com";
  pub const ADGUARD: &str = "unfiltered.adguard-dns.com";
  pub const CONTROLD: &str = "p0.freedns.controld.com";
  pub const COMSS: &str = "dns.comss.one";
  // pub const UNCENSOREDDNS: &str = "anycast.uncensoreddns.org";
}

/// Create Doq clients from HostIp list / 从 HostIp 列表创建 Doq 客户端
pub fn doq_li(li: &[HostIp]) -> Vec<Doq> {
  li.iter().map(|s| Doq::new(s.clone())).collect()
}

/// DoQ server list / DoQ 服务器列表
pub const DOQ_LI: &[HostIp] = &[
  host_ip(dns::ADGUARD, 94, 140, 14, 140),
  host_ip(dns::ADGUARD, 94, 140, 14, 141),
  host_ip(dns::CONTROLD, 76, 76, 2, 11),
  // 阿里 DNS DoQ 对大量 TXT 记录的域名（如 salesforce.com 有 34 条）返回不稳定
  // 有时返回 SERVFAIL，有时只返回部分记录（14-15 条），导致 SPF 记录丢失
  // 普通 UDP DNS 正常，问题出在阿里的 DoQ 实现
  // host_ip(dns::ALIDNS, 223, 5, 5, 5),
  // host_ip(dns::ALIDNS, 223, 6, 6, 6),
  // dns.comss.one 对 AAAA 查询返回假的 SOA 负缓存记录，不返回实际 IPv6 地址
  // 这会导致 SPF 验证 IPv6 地址时失败
  // host_ip(dns::COMSS, 212, 109, 195, 93),
  // host_ip(dns::UNCENSOREDDNS, 91, 239, 100, 100),
];

#[cfg(feature = "static")]
#[static_init::dynamic(lazy)]
pub static DOQ: idns::DnsRace<Doq> = idns::DnsRace::new(doq_li(DOQ_LI));
