//! Machine ID: persistent unique identifier for this machine
//! 机器ID：本机持久化唯一标识

mod error;

use std::{
  fs::{create_dir_all, read_to_string, write},
  path::PathBuf,
  sync::OnceLock,
};

pub use error::Error;

const DIR_NAME: &str = "osid";
const FILE_NAME: &str = "id";
const UNKNOWN: &str = "unknown";

/// Cached result / 缓存结果
static ID: OnceLock<Result<String, Error>> = OnceLock::new();

/// Get data directory path / 获取数据目录路径
pub fn dir() -> PathBuf {
  dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("/etc/id"))
}

/// Initialize machine ID / 初始化机器ID
fn init() -> Result<String, Error> {
  let dir = dir().join(DIR_NAME);
  create_dir_all(&dir).map_err(Error::CreateDir)?;

  let path = dir.join(FILE_NAME);
  if let Ok(id) = read_to_string(&path)
    && !id.is_empty()
  {
    return Ok(id);
  }

  let hostname = hostname::get()
    .map(|s| s.to_string_lossy().into_owned())
    .unwrap_or_else(|_| UNKNOWN.into());
  let id = format!("{hostname}:{}", ub64::u64_b64(rand::random::<u64>()));
  write(&path, &id).map_err(Error::WriteId)?;
  Ok(id)
}

/// Get or create machine ID (cached) / 获取或创建机器ID（缓存）
pub fn get() -> Result<&'static str, &'static Error> {
  ID.get_or_init(init).as_ref().map(|s| s.as_str())
}
