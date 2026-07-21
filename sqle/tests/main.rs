use aok::{OK, Void};
use log::info;
#[cfg(feature = "mysql")]
use sqle::mysql;
#[cfg(feature = "postgres")]
use sqle::postgres;
use sqle::{bool, string};

#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
}

#[test]
fn test() -> Void {
  info!("> test {}", 123456);
  OK
}

#[test]
fn test_string() -> Void {
  // 测试单引号转义
  assert_eq!(string("foo'bar"), r#"'foo''bar'"#);
  assert_eq!(string(""), r#"''"#);

  // 测试换行符转义
  assert_eq!(string("hello\nworld"), r#"'hello\nworld'"#);

  // 测试回车符转义
  assert_eq!(string("hello\rworld"), r#"'hello\rworld'"#);

  // 测试制表符转义
  assert_eq!(string("hello\tworld"), r#"'hello\tworld'"#);

  // 测试反斜杠转义
  assert_eq!(string("hello\\world"), r#"'hello\\world'"#);

  // 测试空字符转义
  assert_eq!(string("hello\0world"), r#"'hello\0world'"#);

  // 测试组合情况
  assert_eq!(string("it's\na\ttest\\"), r#"'it''s\na\ttest\\'"#);

  OK
}

#[test]
fn test_bool() -> Void {
  assert_eq!(bool(true), "TRUE");
  assert_eq!(bool(false), "FALSE");
  OK
}

#[test]
#[cfg(feature = "mysql")]
fn test_mysql() -> Void {
  let bytes = b"hello";
  assert_eq!(mysql::blob(bytes), r#"X'68656C6C6F'"#);
  assert_eq!(mysql::blob(&[]), r#"X''"#);
  OK
}

#[test]
#[cfg(feature = "postgres")]
fn test_postgres() -> Void {
  let bytes = b"hello";
  assert_eq!(postgres::blob(bytes), r#"E'\\x68656c6c6f'"#);
  assert_eq!(postgres::blob(&[]), r#"E'\\x'"#);
  OK
}
