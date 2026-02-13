use std::{env, fs, path::PathBuf};

use super::super::Dns;

pub fn load_conf() -> Dns {
  let path = resolve_path();
  if let Ok(file) = fs::File::open(path) {
    use std::io;
    return Dns::parse_reader(io::BufReader::new(file));
  }
  Dns::default()
}

fn resolve_path() -> PathBuf {
  if let Ok(path) = env::var("RESOLV_CONF") {
    return PathBuf::from(path);
  }
  PathBuf::from("/etc/resolv.conf")
}
