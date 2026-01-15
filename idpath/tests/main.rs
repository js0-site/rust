use std::path::MAIN_SEPARATOR;

use idpath::{DEPTH1, DEPTH2, DEPTH3, decode, encode};

#[test]
fn test_encode_decode() {
  for id in 0..100000 {
    let path = encode("/test", id);
    let decoded = decode(&path).unwrap();
    assert_eq!(id, decoded, "id={id} path={path}");
  }

  // large values / 大数值
  for id in [u64::MAX, u64::MAX - 1, u64::MAX / 2, 1 << 32, 1 << 48] {
    let path = encode("/test", id);
    let decoded = decode(&path).unwrap();
    assert_eq!(id, decoded, "id={id} path={path}");
  }
}

#[test]
fn test_encode_lengths() {
  let cases: &[(u64, usize, &str)] = &[
    // len=0
    (0, 0, "len=0"),
    // len=1: 1-31
    (1, 1, "len=1"),
    (31, 1, "len=1"),
    // len=2: 32-1023
    (32, 2, "len=2"),
    (1023, 2, "len=2"),
    // len=3: 1024-32767
    (1024, 3, "len=3"),
    (32767, 3, "len=3"),
    // len=4: 32768-1048575
    (32768, 4, "len=4"),
    (1048575, 4, "len=4"),
    // len=5: 1048576-33554431
    (1048576, 5, "len=5"),
    (33554431, 5, "len=5"),
    // len=6: 33554432-1073741823
    (33554432, 6, "len>=6"),
    (1073741823, 6, "len>=6"),
    // len>=7
    (1073741824, 7, "len>=7"),
    (u64::MAX, 13, "len>=7"),
  ];

  for &(id, expected_len, desc) in cases {
    let path = encode("/test", id);
    let parts: Vec<&str> = path.as_str().split(MAIN_SEPARATOR).collect();
    println!("{desc}: id={id} -> {path}");

    // Check suffix based on encoded length / 根据编码长度检查后缀
    let last = parts.last().unwrap();
    let last_char = last.chars().last().unwrap();

    match expected_len {
      0..=2 => assert_eq!(last_char, DEPTH1 as char, "len 0-2 should end with DEPTH1"),
      3 => assert_eq!(last_char, DEPTH2 as char, "len 3 should end with DEPTH2"),
      4 => assert_eq!(last_char, DEPTH3 as char, "len 4 should end with DEPTH3"),
      _ => assert!(
        last_char != DEPTH1 as char && last_char != DEPTH2 as char && last_char != DEPTH3 as char,
        "len>=5 should have no suffix"
      ),
    }

    // Verify round-trip / 验证往返
    let decoded = decode(&path).unwrap();
    assert_eq!(id, decoded);
  }
}

#[test]
fn test_path_format() {
  // Test low-bits-first ordering / 测试低位在前排序
  let path = encode("/test", 0x12345);
  let parts: Vec<&str> = path.as_str().split(MAIN_SEPARATOR).collect();
  assert_eq!(parts[0], "");
  assert_eq!(parts[1], "test");
  // 0x12345 = 74565 -> base32 = "29t5" (len=4)
  // low bits first: "t5" / "29" + DEPTH3
  assert_eq!(parts[2], "t5");
  assert!(parts[3].ends_with(DEPTH3 as char));
}
