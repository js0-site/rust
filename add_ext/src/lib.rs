//! Add extension utilities
//! 添加扩展名工具
//!
//! Utilities for appending extensions to file paths.
//! 为文件路径添加扩展名的工具。

use std::{ffi::OsStr, path::PathBuf};

/// Add extension to path
/// 为路径添加扩展名
///
/// If the path already has an extension, appends ".{ext}" to preserve the original extension.
/// 如果路径已有扩展名，则追加 ".{ext}" 以保留原始扩展名。
///
/// # Arguments
/// * `path` - Target path / 目标路径
/// * `ext`  - Extension to add (without dot) / 要添加的扩展名（不含点）
///
/// # Examples
/// ```
/// use std::path::PathBuf;
/// use add_ext::add_ext;
///
/// // Path without extension
/// let path = PathBuf::from("test");
/// assert_eq!(add_ext(&path, "tmp"), PathBuf::from("test.tmp"));
///
/// // Path with extension
/// let path = PathBuf::from("test.json");
/// assert_eq!(add_ext(&path, "tmp"), PathBuf::from("test.json.tmp"));
/// ```
pub fn add_ext(path: impl Into<PathBuf>, ext: impl AsRef<OsStr>) -> PathBuf {
  let mut storage = path.into().into_os_string();
  storage.push(".");
  storage.push(ext);
  PathBuf::from(storage)
}
