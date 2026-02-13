use std::io::{Error, ErrorKind, Result};

use compio_buf::bytes::BufMut;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryType {
  A = 1,
  Aaaa = 28,
}

/// 写入 DNS 问题段
pub fn write_question(name: &str, qtype: QueryType, buf: &mut impl BufMut) -> Result<()> {
  for label in name.split('.') {
    if label.len() > 63 {
      return Err(Error::new(ErrorKind::InvalidInput, "标签过长"));
    }
    buf.put_u8(label.len() as u8);
    buf.put_slice(label.as_bytes());
  }
  buf.put_u8(0);
  buf.put_slice(&(qtype as u16).to_be_bytes());
  buf.put_slice(&1u16.to_be_bytes()); // IN class
  Ok(())
}
