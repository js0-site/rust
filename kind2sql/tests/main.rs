use aok::{OK, Void};

#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
}

#[cfg(feature = "mysql")]
#[test]
fn test_mysql_sql_field() -> Void {
  use kind2sql::{Kind, SqlField, mysql::Mysql};

  // Test multiple types in one go
  // Data format: U8(123) + I16(-456) + String("hello")
  let mut data = vec![];

  // U8: 123
  data.push(123u8);

  // I16: -456 (little endian)
  data.extend_from_slice(&(-456i16).to_le_bytes());

  // String: "hello" (len=5, vb(5)=5)
  data.push(5u8);
  data.extend_from_slice(b"hello");

  let kinds = [Kind::U8, Kind::I16, Kind::String];
  let result = Mysql::sql_field(&kinds, &data[..]).unwrap();

  assert_eq!(result.len(), 3);
  assert_eq!(result[0], "123");
  assert_eq!(result[1], "-456");
  assert_eq!(result[2], "'hello'");

  // Test with Bytes
  let mut data2 = vec![];
  // U32: 999 (使用 vbyte 编码)
  vb::e(999u64, &mut data2);
  // Bytes: [0xAA, 0xBB, 0xCC] (len=3)
  data2.push(3u8);
  data2.extend_from_slice(&[0xAA, 0xBB, 0xCC]);

  let kinds2 = [Kind::U32, Kind::Bytes];
  let result2 = Mysql::sql_field(&kinds2, &data2[..]).unwrap();

  assert_eq!(result2.len(), 2);
  assert_eq!(result2[0], "999");
  assert_eq!(result2[1], "X'AABBCC'");

  OK
}
