//! File ID encoding/decoding utilities
//! 文件 ID 编码/解码工具

use std::path::{Path, PathBuf};

use fast32::base32::CROCKFORD_LOWER;

use crate::ID;

/// Encode id to base32 string
/// 将 id 编码为 base32 字符串
#[inline(always)]
pub fn encode(id: u64) -> String {
  CROCKFORD_LOWER.encode_u64(id)
}

/// Decode base32 string to id
/// 将 base32 字符串解码为 id
#[inline(always)]
pub fn decode(name: &str) -> Option<u64> {
  if name.is_empty() {
    return None;
  }
  CROCKFORD_LOWER.decode_u64(name.as_bytes()).ok()
}

/// Join dir with encoded id
/// 将目录与编码后的 id 拼接
#[inline(always)]
pub fn new(dir: impl AsRef<Path>) -> (ID, PathBuf) {
  let id = crate::id();
  (id, dir.as_ref().join(encode(id)))
}

/// Join dir with encoded id
/// 将目录与编码后的 id 拼接
#[inline(always)]
pub fn id_path(dir: impl AsRef<Path>, id: ID) -> PathBuf {
  dir.as_ref().join(encode(id))
}
