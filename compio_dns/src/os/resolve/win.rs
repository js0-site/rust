use std::{
  net::{IpAddr, SocketAddr},
  path::PathBuf,
};

use rapidhash::RapidHashMap;

use super::super::Dns;

pub fn load_conf() -> Dns {
  let path = resolve_path();
  if path.exists() {
    if let Ok(file) = std::fs::File::open(path) {
      return Dns::parse_reader(std::io::BufReader::new(file));
    }
  }

  // Registry fallback
  load_from_registry().unwrap_or_default()
}

fn resolve_path() -> PathBuf {
  if let Ok(path) = std::env::var("RESOLV_CONF") {
    return PathBuf::from(path);
  }
  let root = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".to_string());
  PathBuf::from(root).join("System32\\drivers\\etc\\resolv.conf")
}

fn load_from_registry() -> Option<Dns> {
  use std::{ffi::OsString, os::windows::ffi::OsStringExt};

  use windows_sys::Win32::System::Registry::{
    ERROR_SUCCESS, HKEY_LOCAL_MACHINE, KEY_READ, LSTATUS, RegCloseKey, RegOpenKeyExW,
    RegQueryValueExW,
  };

  let subkey = encode_wide("SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters");
  let mut hkey = 0;
  let res = unsafe { RegOpenKeyExW(HKEY_LOCAL_MACHINE, subkey.as_ptr(), 0, KEY_READ, &mut hkey) };
  if res != ERROR_SUCCESS as LSTATUS {
    return None;
  }

  // Ensure key is closed
  struct KeyGuard(isize);
  impl Drop for KeyGuard {
    fn drop(&mut self) {
      unsafe { RegCloseKey(self.0) };
    }
  }
  let _guard = KeyGuard(hkey);

  let mut hosts = Dns::default();
  hosts.nameservers.clear();

  // Try NameServer, then DhcpNameServer
  if let Some(ns) =
    query_reg_string(hkey, "NameServer").or_else(|| query_reg_string(hkey, "DhcpNameServer"))
  {
    for addr in ns
      .split_whitespace()
      .filter_map(|s| s.parse::<IpAddr>().ok())
    {
      hosts.nameservers.push(SocketAddr::new(addr, 53));
    }
  }

  // Try SearchList, then DhcpSearchList
  if let Some(search) =
    query_reg_string(hkey, "SearchList").or_else(|| query_reg_string(hkey, "DhcpSearchList"))
  {
    hosts.search = search.split(',').map(|s| s.trim().to_string()).collect();
  }

  if hosts.nameservers.is_empty() {
    None
  } else {
    Some(hosts)
  }
}

fn query_reg_string(hkey: isize, name: &str) -> Option<String> {
  use windows_sys::Win32::System::Registry::{ERROR_SUCCESS, RegQueryValueExW};

  let name_wide = encode_wide(name);
  let mut len = 0;
  unsafe {
    let mut res = RegQueryValueExW(
      hkey,
      name_wide.as_ptr(),
      std::ptr::null_mut(),
      std::ptr::null_mut(),
      std::ptr::null_mut(),
      &mut len,
    );
    if res != ERROR_SUCCESS as i32 {
      return None;
    }

    let mut buf = vec![0u16; (len / 2) as usize];
    res = RegQueryValueExW(
      hkey,
      name_wide.as_ptr(),
      std::ptr::null_mut(),
      std::ptr::null_mut(),
      buf.as_mut_ptr() as *mut u8,
      &mut len,
    );
    if res == ERROR_SUCCESS as i32 {
      let s = String::from_utf16_lossy(&buf);
      return Some(s.trim_matches('\0').to_string());
    }
  }
  None
}

fn encode_wide(s: &str) -> Vec<u16> {
  use std::os::windows::ffi::OsStrExt;
  std::ffi::OsStr::new(s)
    .encode_wide()
    .chain(std::iter::once(0))
    .collect()
}
