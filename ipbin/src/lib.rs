use std::net::IpAddr;

pub fn ipbin(ipaddr: IpAddr) -> Vec<u8> {
  match ipaddr {
    IpAddr::V4(ipv4) => ipv4.octets().to_vec(),
    IpAddr::V6(ipv6) => ipv6.octets().to_vec(),
  }
}
