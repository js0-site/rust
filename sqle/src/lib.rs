#![cfg_attr(docsrs, feature(doc_cfg))]

/// SQL 字符串转义：单引号、反斜杠、换行符等特殊字符
pub fn string(s: impl AsRef<[u8]>) -> String {
  let s_bytes = s.as_ref();
  // 预估容量：最坏情况下每个字节可能需要转义成两个字节，再加上两个引号
  let mut buf = Vec::with_capacity(s_bytes.len() * 2 + 2);
  buf.push(b'\'');
  for &b in s_bytes {
    match b {
      b'\'' => buf.extend_from_slice(b"''"),   // 单引号
      b'\\' => buf.extend_from_slice(b"\\\\"), // 反斜杠
      b'\n' => buf.extend_from_slice(b"\\n"),  // 换行
      b'\r' => buf.extend_from_slice(b"\\r"),  // 回车
      b'\t' => buf.extend_from_slice(b"\\t"),  // 制表符
      b'\0' => buf.extend_from_slice(b"\\0"),  // 空字符
      _ => buf.push(b),
    }
  }
  buf.push(b'\'');
  unsafe { String::from_utf8_unchecked(buf) }
}

/// MYSQL 二进制，X'HEX'
#[cfg(feature = "mysql")]
pub mod mysql {
  pub fn blob(bytes: &[u8]) -> String {
    let len = 3 + bytes.len() * 2;
    let mut buf = vec![0; len];
    buf[0] = b'X';
    buf[1] = b'\'';
    buf[len - 1] = b'\'';
    if !bytes.is_empty() {
      faster_hex::hex_encode_upper(bytes, &mut buf[2..len - 1]).unwrap();
    }
    unsafe { String::from_utf8_unchecked(buf) }
  }
}

/// POSTGRESQL 二进制，E'\\xHEX'
#[cfg(feature = "postgres")]
pub mod postgres {
  pub fn blob(bytes: &[u8]) -> String {
    let len = 6 + bytes.len() * 2;
    let mut buf = vec![0; len];
    buf[0] = b'E';
    buf[1] = b'\'';
    buf[2] = b'\\';
    buf[3] = b'\\';
    buf[4] = b'x';
    buf[len - 1] = b'\'';
    if !bytes.is_empty() {
      faster_hex::hex_encode(bytes, &mut buf[5..len - 1]).unwrap();
    }
    unsafe { String::from_utf8_unchecked(buf) }
  }
}

pub fn bool(b: bool) -> &'static str {
  if b { "TRUE" } else { "FALSE" }
}
