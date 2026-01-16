#![cfg_attr(docsrs, feature(doc_cfg))]

mod dot;
mod error;

use std::net::{IpAddr, Ipv4Addr};

pub use dot::Dot;
pub use error::{Error, Result};
use hipstr::HipStr;
use idns::Answer;
pub use idns::QType;

/// Host with IP / 主机与 IP
#[derive(Debug, Clone)]
pub struct HostIp {
  pub host: HipStr<'static>,
  pub ip: IpAddr,
}

impl idns::Query for Dot {
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

/// DoT server hostnames / DoT 服务器域名
pub mod dns {
  pub const CLOUDFLARE: &str = "cloudflare-dns.com";
  pub const GOOGLE: &str = "dns.google";
  pub const QUAD9: &str = "dns.quad9.net";
  /// 360 安全 DNS（中国）
  pub const DNS360: &str = "dot.360.cn";
  /// TWNIC（台湾）- 证书只对 IP 有效
  pub const TWNIC: &str = "101.101.101.101";
  /// IIJ DNS（日本）
  pub const IIJ: &str = "public.dns.iij.jp";
  /// 阿里 DNS（中国）
  pub const ALIDNS: &str = "dns.alidns.com";
  /// 腾讯 DNSPod（中国）
  pub const DNSPOD: &str = "dot.pub";
}

/// Create Dot clients from HostIp list / 从 HostIp 列表创建 Dot 客户端
pub fn dot_li(li: &[HostIp]) -> Vec<Dot> {
  li.iter().map(|s| Dot::new(s.clone())).collect()
}

/// DoT server list / DoT 服务器列表
pub const DOT_LI: &[HostIp] = &[
  host_ip(dns::CLOUDFLARE, 1, 1, 1, 1),
  host_ip(dns::CLOUDFLARE, 1, 0, 0, 1),
  host_ip(dns::GOOGLE, 8, 8, 8, 8),
  host_ip(dns::GOOGLE, 8, 8, 4, 4),
  host_ip(dns::QUAD9, 9, 9, 9, 9),
  // 360 安全 DNS（中国，无过滤）
  host_ip(dns::DNS360, 101, 226, 4, 6),
  host_ip(dns::DNS360, 218, 30, 118, 6),
  // TWNIC（台湾）
  host_ip(dns::TWNIC, 101, 101, 101, 101),
  // IIJ DNS（日本）
  host_ip(dns::IIJ, 103, 2, 57, 5),
  // 阿里 DNS：对大量 TXT 记录（如 salesforce.com 34 条）返回 0 条或不完整
  // host_ip(dns::ALIDNS, 223, 5, 5, 5),
  // host_ip(dns::ALIDNS, 223, 6, 6, 6),
  // 腾讯 DNSPod：1.12.12.12 返回 early eof，120.53.53.53 只返回 4 条 TXT
  // host_ip(dns::DNSPOD, 1, 12, 12, 12),
  // host_ip(dns::DNSPOD, 120, 53, 53, 53),
];

#[cfg(feature = "static")]
#[static_init::dynamic(lazy)]
pub static DOT: idns::DnsRace<Dot> = idns::DnsRace::new(dot_li(DOT_LI));
