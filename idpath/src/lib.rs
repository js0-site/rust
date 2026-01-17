//! Path encoding utilities / 路径编码工具
//!
//! Encodes IDs into hierarchical paths like `xx/xx/xx` (low bits first) to improve
//! load balancing in storage systems.
//!
//! 将 ID 编码为 `xx/xx/xx` 格式的路径（低位在前），有利于在存储系统中实现负载均衡。

mod error;

use std::path::MAIN_SEPARATOR;

pub use error::{Error, Result};
use fast32::base32::CROCKFORD_LOWER;
use hipstr::HipStr;

/// Depth suffixes / 深度后缀
pub const DEPTH1: u8 = b'_'; // len 0-2: 1 level
pub const DEPTH2: u8 = b'-'; // len 3: 2 levels
pub const DEPTH3: u8 = b'~'; // len 4: 2 levels

const SEP: u8 = MAIN_SEPARATOR as u8;

/// Encode id to path: prefix/xx/xx/xx (low bits first)
/// 编码 ID 为路径（低位在前）
#[inline]
pub fn encode(prefix: impl AsRef<str>, id: u64) -> HipStr<'static> {
  let prefix = prefix.as_ref();
  let encoded = CROCKFORD_LOWER.encode_u64(id);
  let enc = encoded.as_bytes();
  let len = enc.len();

  let cap = prefix.len() + (len + 3).max(6);
  let mut buf = Vec::with_capacity(cap);

  buf.extend_from_slice(prefix.as_bytes());
  buf.push(SEP);

  match len {
    0..=2 => {
      buf.extend_from_slice(enc);
      buf.push(DEPTH1);
    }
    3 => {
      buf.extend_from_slice(&enc[1..3]);
      buf.push(SEP);
      buf.push(enc[0]);
      buf.push(DEPTH2);
    }
    4 => {
      buf.extend_from_slice(&enc[2..4]);
      buf.push(SEP);
      buf.extend_from_slice(&enc[..2]);
      buf.push(DEPTH3);
    }
    5 => {
      buf.extend_from_slice(&enc[3..5]);
      buf.push(SEP);
      buf.extend_from_slice(&enc[1..3]);
      buf.push(SEP);
      buf.push(enc[0]);
    }
    _ => {
      buf.extend_from_slice(&enc[len - 2..]);
      buf.push(SEP);
      buf.extend_from_slice(&enc[len - 4..len - 2]);
      buf.push(SEP);
      buf.extend_from_slice(&enc[..len - 4]);
    }
  }

  // SAFETY: all bytes are valid UTF-8 (ASCII from Crockford base32 and separator)
  unsafe { HipStr::from_utf8_unchecked(buf.into()) }
}

/// Decode path to id / 解码路径为 ID
pub fn decode(path: impl AsRef<str>) -> Result<u64> {
  let path_str = path.as_ref();
  let mut iter = path_str.rsplit(MAIN_SEPARATOR);
  let last_part = iter
    .next()
    .ok_or_else(|| Error::InvalidPath(path_str.into()))?;

  let mut buf = [0u8; 16];
  let mut pos = 0;

  let mut push = |src: &[u8]| -> Result<()> {
    let len = src.len();
    if pos + len > buf.len() {
      return Err(Error::InvalidPath(path_str.into()));
    }
    buf[pos..pos + len].copy_from_slice(src);
    pos += len;
    Ok(())
  };

  if let Some(&suffix) = last_part.as_bytes().last() {
    match suffix {
      DEPTH1 => {
        let name = &last_part[..last_part.len() - 1];
        push(name.as_bytes())?;
      }
      DEPTH2 | DEPTH3 => {
        let name = &last_part[..last_part.len() - 1];
        let d2 = iter
          .next()
          .ok_or_else(|| Error::InvalidPath(path_str.into()))?;
        if d2.len() != 2 {
          return Err(Error::InvalidPath(path_str.into()));
        }
        push(name.as_bytes())?;
        push(d2.as_bytes())?;
      }
      _ => {
        let d2 = iter
          .next()
          .ok_or_else(|| Error::InvalidPath(path_str.into()))?;
        let d1 = iter
          .next()
          .ok_or_else(|| Error::InvalidPath(path_str.into()))?;
        if d2.len() != 2 || d1.len() != 2 {
          return Err(Error::InvalidPath(path_str.into()));
        }
        push(last_part.as_bytes())?;
        push(d2.as_bytes())?;
        push(d1.as_bytes())?;
      }
    }
  } else {
    return Err(Error::InvalidPath(path_str.into()));
  }

  CROCKFORD_LOWER
    .decode_u64(&buf[..pos])
    .map_err(|_| Error::DecodeFailed(path_str.into()))
}
