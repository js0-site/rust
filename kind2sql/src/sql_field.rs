use bytes::buf::Buf;

use crate::Kind;

fn sql_field(
  blob: impl Fn(&[u8]) -> String,
  kind: Kind,
  mut buf: impl Buf,
  out: &mut Vec<String>,
) -> vb::Result<()> {
  use Kind::*;
  out.push(match kind {
    Bool => if buf.get_u8() == 0 { 0 } else { 1 }.to_string(),
    U8 => buf.get_u8().to_string(),
    I8 => buf.get_i8().to_string(),
    U16 => buf.get_u16_le().to_string(),
    I16 => buf.get_i16_le().to_string(),
    U32 | U64 => {
      // U32 和 U64 使用 vbyte 编码
      let (value, consumed) = vb::d(buf.chunk())?;
      buf.advance(consumed);
      value.to_string()
    }
    I32 => buf.get_i32_le().to_string(),
    I64 => buf.get_i64_le().to_string(),
    String | Bytes => {
      // Bytes , String 编码 = vbyte编码的长度, 内容
      let (len, consumed) = vb::d(buf.chunk())?;
      buf.advance(consumed);
      let len = len as usize;
      let chunk = buf.chunk();
      let data = &chunk[..len];
      let result = match kind {
        String => sqle::string(data),
        Bytes => blob(data),
        _ => unreachable!(),
      };
      buf.advance(len);
      result
    }
  });
  Ok(())
}

pub trait SqlField {
  fn blob(data: &[u8]) -> String;

  fn sql_field(kind_li: &[Kind], mut buf: impl Buf) -> vb::Result<Vec<String>> {
    let mut out = Vec::with_capacity(kind_li.len());
    for &kind in kind_li {
      sql_field(Self::blob, kind, &mut buf, &mut out)?;
    }
    Ok(out)
  }
}
