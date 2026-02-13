use std::{fs, net::IpAddr};

use rapidhash::RapidHashMap;

use super::hosts_path;

pub(crate) fn load_hosts() -> RapidHashMap<String, IpAddr> {
  let mut hosts = RapidHashMap::default();
  let path = hosts_path();

  if let Ok(file) = fs::File::open(path) {
    use std::io::{BufRead, BufReader};
    let reader = BufReader::new(file);

    for line in reader.lines().map_while(Result::ok) {
      let line = line
        .split_once('#')
        .map_or(line.as_str(), |(s, _)| s)
        .trim();
      if line.is_empty() {
        continue;
      }
      let mut parts = line.split_whitespace();
      if let Some(addr_str) = parts.next()
        && let Ok(addr) = addr_str.parse::<IpAddr>()
      {
        for host in parts {
          // keys in hosts file are case-insensitive
          hosts.entry(host.to_lowercase()).or_insert(addr);
        }
      }
    }
  }
  hosts
}
