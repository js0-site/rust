use std::{
  cmp, io,
  net::{IpAddr, Ipv4Addr, SocketAddr},
  time::Duration,
};

use super::super::{Dns, resolve::load_conf};

impl Dns {
  pub fn load() -> io::Result<Self> {
    Ok(load_conf())
  }

  pub fn parse(content: &str) -> Self {
    Self::parse_reader(io::Cursor::new(content.as_bytes()))
  }

  pub fn parse_reader<R: io::BufRead>(reader: R) -> Self {
    let mut nameservers = Vec::new();
    let mut search = Vec::new();
    let mut ndots = 1u8;
    let mut timeout = Duration::from_secs(5);
    let mut attempts = 2u8;

    for line in reader.lines().map_while(Result::ok) {
      let line = line
        .split_once('#')
        .map_or(line.as_str(), |(s, _)| s)
        .trim();
      if line.is_empty() {
        continue;
      }

      let mut parts = line.split_whitespace();
      let Some(key) = parts.next() else { continue };

      match key {
        "nameserver" => {
          if let Some(value) = parts.next()
            && let Ok(ip) = value.parse::<IpAddr>()
          {
            nameservers.push(SocketAddr::new(ip, 53));
          }
        }
        "search" | "domain" => {
          search.clear();
          search.extend(parts.map(|s| s.to_string()));
        }
        "options" => {
          for opt in parts {
            if let Some(v) = opt.strip_prefix("ndots:")
              && let Ok(n) = v.parse::<u8>()
            {
              ndots = cmp::min(n, 15);
            } else if let Some(v) = opt.strip_prefix("timeout:")
              && let Ok(n) = v.parse::<u64>()
            {
              timeout = Duration::from_secs(cmp::min(n, 30));
            } else if let Some(v) = opt.strip_prefix("attempts:")
              && let Ok(n) = v.parse::<u8>()
            {
              attempts = cmp::min(n, 5);
            }
          }
        }
        _ => {}
      }
    }

    if nameservers.is_empty() {
      nameservers.push(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 53));
    }

    Self {
      nameservers,
      search,
      ndots,
      timeout,
      attempts,
    }
  }
}
