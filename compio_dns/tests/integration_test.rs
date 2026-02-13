use std::net::SocketAddr;

use compio_dns::Resolve;

#[compio::test]
async fn test_lookup_google() {
  let resolver = Resolve::new().unwrap();
  let addrs = resolver.lookup("google.com").await.unwrap();
  let addrs: Vec<SocketAddr> = addrs.collect();
  assert!(!addrs.is_empty());
}

#[compio::test]
async fn test_lookup_localhost() {
  let resolver = Resolve::new().unwrap();
  let addrs = resolver.lookup("localhost").await.unwrap();
  let addrs: Vec<SocketAddr> = addrs.collect();
  assert!(!addrs.is_empty());
  // localhost usually resolves to 127.0.0.1 or ::1
  assert!(addrs.iter().any(|a| a.ip().is_loopback()));
}
