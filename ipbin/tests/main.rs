use std::net::IpAddr;

use aok::{OK, Void};
use ipbin::ipbin;
use log::info;

#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
}

#[test]
fn test_ipv4() -> Void {
  let ip: IpAddr = "127.0.0.1".parse().unwrap();
  let result = ipbin(ip);
  assert_eq!(result, vec![127, 0, 0, 1]);
  info!("IPv4 test passed: 127.0.0.1 -> {:?}", result);
  OK
}

#[test]
fn test_ipv4_zero() -> Void {
  let ip: IpAddr = "0.0.0.0".parse().unwrap();
  let result = ipbin(ip);
  assert_eq!(result, vec![0, 0, 0, 0]);
  info!("IPv4 zero test passed: 0.0.0.0 -> {:?}", result);
  OK
}

#[test]
fn test_ipv4_max() -> Void {
  let ip: IpAddr = "255.255.255.255".parse().unwrap();
  let result = ipbin(ip);
  assert_eq!(result, vec![255, 255, 255, 255]);
  info!("IPv4 max test passed: 255.255.255.255 -> {:?}", result);
  OK
}

#[test]
fn test_ipv6_loopback() -> Void {
  let ip: IpAddr = "::1".parse().unwrap();
  let result = ipbin(ip);
  assert_eq!(result, vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]);
  info!("IPv6 loopback test passed: ::1 -> {:?}", result);
  OK
}

#[test]
fn test_ipv6_full() -> Void {
  let ip: IpAddr = "2001:0db8:85a3:0000:0000:8a2e:0370:7334".parse().unwrap();
  let result = ipbin(ip);
  assert_eq!(
    result,
    vec![
      0x20, 0x01, 0x0d, 0xb8, 0x85, 0xa3, 0x00, 0x00, 0x00, 0x00, 0x8a, 0x2e, 0x03, 0x70, 0x73,
      0x34
    ]
  );
  info!(
    "IPv6 full test passed: 2001:0db8:85a3:0000:0000:8a2e:0370:7334 -> {:?}",
    result
  );
  OK
}

#[test]
fn test_ipv6_zero() -> Void {
  let ip: IpAddr = "::".parse().unwrap();
  let result = ipbin(ip);
  assert_eq!(result, vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
  info!("IPv6 zero test passed: :: -> {:?}", result);
  OK
}
