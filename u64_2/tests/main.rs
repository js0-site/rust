use aok::{OK, Void};
use u64_2::{decode, encode};

#[test]
fn test_encode_decode() -> Void {
  let mut buffer = [0u8; 32]; // 缓冲区稍微大一点，防止越界读写
  let num1: u64 = 500; // 需要 2 字节
  let num2: u64 = 100000; // 需要 3 字节

  println!("原始数据: {}, {}", num1, num2);

  // 编码
  let len = encode(num1, num2, &mut buffer);
  println!("编码后长度: {} 字节", len);
  println!("Hex: {:02X?}", &buffer[..len]);

  // 解码
  let (d1, d2, consumed) = decode(&buffer);
  println!("解码结果: {}, {}", d1, d2);
  assert_eq!(num1, d1);
  assert_eq!(num2, d2);
  assert_eq!(len, consumed);

  OK
}
