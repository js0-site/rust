use std::net::{IpAddr, Ipv4Addr};

#[derive(Debug, Clone, Copy)]
pub struct DnsServer {
  pub ip: IpAddr,
}

const fn ip(a: u8, b: u8, c: u8, d: u8) -> DnsServer {
  DnsServer {
    ip: IpAddr::V4(Ipv4Addr::new(a, b, c, d)),
  }
}

pub const DNS_SERVER_LI: [DnsServer; 8] = [
  ip(8, 8, 8, 8),         // Google
  ip(1, 1, 1, 1),         // Cloudflare
  ip(223, 5, 5, 5),       // AliDNS
  ip(208, 67, 222, 222),  // OpenDNS
  ip(9, 9, 9, 9),         // Quad9
  ip(1, 0, 0, 1),         // Cloudflare
  ip(114, 114, 114, 114), // 114DNS
  ip(180, 76, 76, 76),    // Baidu
];
