use std::{collections::HashMap, io, net::IpAddr};

pub(crate) fn load_hosts() -> io::Result<HashMap<String, IpAddr>> {
  let content =
    std::fs::read_to_string("/etc/hosts").or_else(|_| Ok::<_, io::Error>(String::new()))?;
  let mut hosts = HashMap::new();
  for line in content.lines() {
    let line = line.split('#').next().unwrap_or("").trim();
    if line.is_empty() {
      continue;
    }
    let mut parts = line.split_whitespace();
    if let Some(addr_str) = parts.next()
      && let Ok(addr) = addr_str.parse::<IpAddr>()
    {
      for host in parts {
        hosts.insert(host.to_lowercase(), addr);
      }
    }
  }
  Ok(hosts)
}
