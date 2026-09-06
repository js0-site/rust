use aok::{OK, Void};
use log::info;

#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
}

#[test]
fn test() -> Void {
  info!("> test {}", 123456);
  OK
}

/// Test empty strings
/// 测试空字符串
#[test]
fn test_empty_strings() -> Void {
  assert_eq!(shared_prefix_len::shared_prefix_len(b"", b""), 0);
  assert_eq!(shared_prefix_len::shared_prefix_len(b"abc", b""), 0);
  assert_eq!(shared_prefix_len::shared_prefix_len(b"", b"abc"), 0);
  OK
}

/// Test identical strings
/// 测试相同字符串
#[test]
fn test_identical_strings() -> Void {
  assert_eq!(shared_prefix_len::shared_prefix_len(b"hello", b"hello"), 5);
  assert_eq!(shared_prefix_len::shared_prefix_len(b"", b""), 0);
  assert_eq!(shared_prefix_len::shared_prefix_len(b"a", b"a"), 1);
  OK
}

/// Test completely different strings
/// 测试完全不同的字符串
#[test]
fn test_completely_different() -> Void {
  assert_eq!(shared_prefix_len::shared_prefix_len(b"abc", b"xyz"), 0);
  assert_eq!(shared_prefix_len::shared_prefix_len(b"hello", b"world"), 0);
  assert_eq!(shared_prefix_len::shared_prefix_len(b"a", b"b"), 0);
  OK
}

/// Test partial prefix matches
/// 测试部分前缀匹配
#[test]
fn test_partial_prefix() -> Void {
  assert_eq!(shared_prefix_len::shared_prefix_len(b"hello", b"help"), 3);
  assert_eq!(shared_prefix_len::shared_prefix_len(b"prefix", b"pre"), 3);
  assert_eq!(shared_prefix_len::shared_prefix_len(b"abc", b"abcd"), 3);
  assert_eq!(shared_prefix_len::shared_prefix_len(b"abcd", b"abc"), 3);
  OK
}

/// Test single byte differences
/// 测试单字节差异
#[test]
fn test_single_byte_difference() -> Void {
  assert_eq!(shared_prefix_len::shared_prefix_len(b"hello", b"hallo"), 1);
  assert_eq!(shared_prefix_len::shared_prefix_len(b"abcde", b"abXde"), 2);
  assert_eq!(shared_prefix_len::shared_prefix_len(b"12345", b"123X5"), 3);
  OK
}

/// Test different lengths
/// 测试不同长度
#[test]
fn test_different_lengths() -> Void {
  assert_eq!(
    shared_prefix_len::shared_prefix_len(b"short", b"shorter"),
    5
  );
  assert_eq!(shared_prefix_len::shared_prefix_len(b"longer", b"long"), 4);
  assert_eq!(shared_prefix_len::shared_prefix_len(b"a", b"ab"), 1);
  assert_eq!(shared_prefix_len::shared_prefix_len(b"ab", b"a"), 1);
  OK
}

/// Test binary data
/// 测试二进制数据
#[test]
fn test_binary_data() -> Void {
  assert_eq!(
    shared_prefix_len::shared_prefix_len(&[0, 1, 2, 3], &[0, 1, 2, 4]),
    3
  );
  assert_eq!(
    shared_prefix_len::shared_prefix_len(&[255, 254, 253], &[255, 254, 252]),
    2
  );
  assert_eq!(
    shared_prefix_len::shared_prefix_len(&[0x00, 0xFF], &[0x00, 0xFF]),
    2
  );
  OK
}

/// Test multi-byte chunks (u64 alignment)
/// 测试多字节块（u64 对齐）
#[test]
fn test_multi_byte_chunks() -> Void {
  // Test strings longer than 8 bytes
  assert_eq!(
    shared_prefix_len::shared_prefix_len(b"abcdefghijk", b"abcdefghxyz"),
    8
  );
  assert_eq!(
    shared_prefix_len::shared_prefix_len(b"1234567890", b"12345678ab"),
    8
  );

  // Test difference in first chunk
  assert_eq!(
    shared_prefix_len::shared_prefix_len(b"abcdefgh", b"abcdefff"),
    6
  );

  // Test difference across chunks
  assert_eq!(
    shared_prefix_len::shared_prefix_len(b"abcdefghijklmnopqrst", b"abcdefghijklmnxxxx"),
    14
  );
  OK
}

/// Test boundary conditions
/// 测试边界条件
#[test]
fn test_boundary_conditions() -> Void {
  // Exactly 8 bytes
  assert_eq!(
    shared_prefix_len::shared_prefix_len(b"12345678", b"12345678"),
    8
  );

  // 7 bytes vs 8 bytes
  assert_eq!(
    shared_prefix_len::shared_prefix_len(b"1234567", b"12345678"),
    7
  );

  // 9 bytes vs 10 bytes
  assert_eq!(
    shared_prefix_len::shared_prefix_len(b"123456789", b"1234567890"),
    9
  );
  OK
}

/// Test performance with large data
/// 测试大数据量性能
#[test]
fn test_large_data() -> Void {
  // Create two large byte arrays with shared prefix
  let mut a = vec![0u8; 1024];
  let mut b = vec![0u8; 1024];

  // First 500 bytes are identical
  for i in 0..500 {
    a[i] = (i % 256) as u8;
    b[i] = (i % 256) as u8;
  }

  // Rest are different
  a[500] = 0;
  b[500] = 255;

  assert_eq!(shared_prefix_len::shared_prefix_len(&a, &b), 500);
  OK
}

/// Test unicode/utf-8 byte patterns
/// 测试 unicode/utf-8 字节模式
#[test]
fn test_utf8_patterns() -> Void {
  // Common Chinese characters (3-byte UTF-8 sequences)
  let a = "你好世界".as_bytes();
  let b = "你好世界abc".as_bytes();
  assert_eq!(shared_prefix_len::shared_prefix_len(a, b), 12); // "你好世界" is 12 bytes

  let c = "你好测试".as_bytes();
  assert_eq!(shared_prefix_len::shared_prefix_len(a, c), 6); // "你好" is 6 bytes
  OK
}
