#[cfg(feature = "bitcode")]
use jdb_xorf::{Bf, Bf8};

// 全局常量字符串
#[cfg(feature = "bitcode")]
const KEYS: &[&str] = &["apple", "banana", "cherry"];

#[cfg(not(feature = "bitcode"))]
fn main() {
  println!("Please run with --features bitcode");
}

#[cfg(feature = "bitcode")]
fn main() {
  fn create_and_encode() -> Vec<u8> {
    println!("1. Creating and encoding filter...");

    // 直接使用 &str 切片，无需转换为 String
    // Use &str slice directly, no need to convert to String
    let bf: Bf<&str, Bf8> = Bf::from(KEYS);

    // 直接使用 bitcode 库函数序列化
    // Direct usage of bitcode library function for serialization
    // jdb_xorf::Bf supports bitcode::Encode
    let bytes = bitcode::encode(&bf);

    println!("   Bf<&str, Bf8> created and encoded.");
    bytes
  }

  fn decode_and_query(data: &[u8]) {
    println!("2. Decoding filter...");

    // 直接使用 bitcode 库函数反序列化
    // Direct usage of bitcode library function for deserialization
    // jdb_xorf::Bf supports bitcode::Decode
    // 注意：这里我们恢复为 Bf<&str> 类型，PhantomData 可以处理生命周期
    // Note: Here we restore to Bf<&str> type, PhantomData handles the lifetime
    let bf_string: Bf<&str, Bf8> = bitcode::decode(data).expect("Failed to decode filter");

    println!("   Bf<&str, Bf8> decoded successfully.");

    // 验证
    for key in KEYS {
      if bf_string.has(*key) {
        println!("Filter contains '{}': YES", key);
      } else {
        println!("Filter contains '{}': NO (Error!)", key);
      }
    }

    let unknown = "durian";
    if bf_string.has(unknown) {
      println!("Filter contains '{}': YES (False Positive)", unknown);
    } else {
      println!("Filter contains '{}': NO", unknown);
    }
  }
  // 1. 序列化流程 (Encode)
  let encoded_data = create_and_encode();
  println!("Encoded size: {} bytes", encoded_data.len());

  // 2. 反序列化与还原流程 (Decode)
  decode_and_query(&encoded_data);
}
