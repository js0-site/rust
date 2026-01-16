//! Tests for add_ext module
//! add_ext 模块测试

use std::path::PathBuf;

use add_ext::add_ext;

#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
}

#[test]
fn test_add_ext_no_ext() {
  // Path without extension
  // 无扩展名的路径
  let path = PathBuf::from("test");
  assert_eq!(add_ext(&path, "tmp"), PathBuf::from("test.tmp"));
}

#[test]
fn test_add_ext_with_ext() {
  // Path with extension
  // 有扩展名的路径
  let path = PathBuf::from("test.json");
  assert_eq!(add_ext(&path, "tmp"), PathBuf::from("test.json.tmp"));
}

#[test]
fn test_add_ext_nested_path() {
  // Nested path
  // 嵌套路径
  let path = PathBuf::from("dir/subdir/file.txt");
  assert_eq!(
    add_ext(&path, "bak"),
    PathBuf::from("dir/subdir/file.txt.bak")
  );
}

#[test]
fn test_add_ext_empty_ext() {
  // Empty extension
  // 空扩展名
  let path = PathBuf::from("file");
  assert_eq!(add_ext(&path, ""), PathBuf::from("file."));
}

#[test]
fn test_add_ext_multiple_dots() {
  // Path with multiple dots
  // 多点路径
  let path = PathBuf::from("archive.tar.gz");
  assert_eq!(add_ext(&path, "tmp"), PathBuf::from("archive.tar.gz.tmp"));
}
