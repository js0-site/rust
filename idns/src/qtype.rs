use std::fmt::{self, Display, Formatter};

/// DNS 记录类型 / DNS record type
#[derive(Debug, Clone, Copy, Default)]
#[repr(u16)]
#[allow(non_camel_case_types)]
pub enum QType {
  #[default]
  A = 1,
  NS = 2,
  MD = 3, // 过时
  MF = 4, // 过时
  CNAME = 5,
  SOA = 6,
  MB = 7,
  MG = 8,
  MR = 9,
  NULL = 10, // 过时
  WKS = 11,
  PTR = 12,
  HINFO = 13,
  MINFO = 14,
  MX = 15,
  TXT = 16,
  RP = 17,
  AFSDB = 18,
  X25 = 19,
  ISDN = 20,
  RT = 21,
  NSAP = 22,
  NSAP_PTR = 23,
  SIG = 24,
  KEY = 25,
  PX = 26,
  GPOS = 27,
  AAAA = 28,
  LOC = 29,
  NXT = 30, // 过时
  EID = 31,
  NIMLOC = 32,
  SRV = 33,
  ATMA = 34,
  NAPTR = 35,
  KX = 36,
  CERT = 37,
  A6 = 38, // 实验性
  DNAME = 39,
  SINK = 40,
  OPT = 41, // EDNS0 选项
  APL = 42,
  DS = 43,
  SSHFP = 44,
  IPSECKEY = 45,
  RRSIG = 46,
  NSEC = 47,
  DNSKEY = 48,
  DHCID = 49,
  NSEC3 = 50,
  NSEC3PARAM = 51,
  TLSA = 52,
  SMIMEA = 53,
  UNUSED = 54, // 未使用
  HIP = 55,
  NINFO = 56,
  RKEY = 57,
  TALINK = 58,
  CDS = 59,
  CDNSKEY = 60,
  OPENPGPKEY = 61,
  CSYNC = 62,
  ZONEMD = 63,
  SVCB = 64,
  HTTPS = 65,
  SPF = 99,
  UINFO = 100,  // 保留
  UID = 101,    // 保留
  GID = 102,    // 保留
  UNSPEC = 103, // 保留
  NID = 104,
  L32 = 105,
  L64 = 106,
  LP = 107,
  EUI48 = 108,
  EUI64 = 109,
  TKEY = 249,
  TSIG = 250,
  IXFR = 251,
  AXFR = 252,
  MAILB = 253, // 过时
  MAILA = 254, // 过时
  ANY = 255,   // 所有记录
}

impl Display for QType {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}
