use std::{net::IpAddr, path::PathBuf};

use rapidhash::RapidHashMap;

pub(crate) mod load;

#[static_init::dynamic]
pub(crate) static HOST_IP: RapidHashMap<String, IpAddr> = load::load_hosts();

pub fn hosts_path() -> PathBuf {
  #[cfg(not(windows))]
  {
    PathBuf::from("/etc/hosts")
  }
  #[cfg(windows)]
  {
    let root = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".to_string());
    PathBuf::from(root).join("System32\\drivers\\etc\\hosts")
  }
}
