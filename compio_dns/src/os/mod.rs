mod hosts;
mod resolve;

use std::{
  net::{IpAddr, Ipv4Addr, SocketAddr},
  time::Duration,
};

pub(crate) use hosts::HOST_IP;

#[derive(Debug, Clone)]
pub struct Dns {
  pub nameservers: Vec<SocketAddr>,
  pub search: Vec<String>,
  pub ndots: u8,
  pub timeout: Duration,
  pub attempts: u8,
}

impl Default for Dns {
  fn default() -> Self {
    Self {
      nameservers: vec![SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 53)],
      search: Vec::new(),
      ndots: 1,
      timeout: Duration::from_secs(5),
      attempts: 2,
    }
  }
}

#[static_init::dynamic]
pub(crate) static DNS: Dns = Dns::load().unwrap_or_else(|_| {
  // If loading fails (e.g. config file missing), fall back to default (usually localhost)
  Dns::parse("")
});
