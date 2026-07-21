// 测试 bitcode 0.6.9 的正确用法

#[derive(Debug, bitcode::Encode, bitcode::Decode)]
struct TestStruct {
  value: u32,
  name: String,
}

fn main() {
  let test = TestStruct {
    value: 42,
    name: "hello".to_string(),
  };

  // 尝试编码
  let encoded = bitcode::encode(&test);
  println!("Encoded: {:?}", encoded);

  // 尝试解码
  let decoded: TestStruct = bitcode::decode(&encoded).unwrap();
  println!("Decoded: {:?}", decoded);
}
