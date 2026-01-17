pub use dns_parse::parse;

/// RFC 9250: DNS message ID MUST be 0 for DoQ
pub fn build(domain: &str, qtype: u16) -> bytes::Bytes {
  dns_parse::build(0, domain, qtype)
}
