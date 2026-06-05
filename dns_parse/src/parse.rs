use std::{
  net::{Ipv4Addr, Ipv6Addr},
  str,
};

use bytes::{Buf, BufMut, Bytes, BytesMut};
use idns::Answer;

use crate::{Error, Result};

/// 解析 DNS 响应 / Parse DNS response
pub fn parse(data: Bytes) -> Result<Vec<Answer>> {
  if data.len() < 12 {
    return Err(Error::ResponseTooShort);
  }

  // Check RCODE (lower 4 bits of byte 3)
  // 0=NOERROR, 1=FORMERR, 2=SERVFAIL, 3=NXDOMAIN, etc.
  let rcode = data[3] & 0x0F;
  if rcode != 0 && rcode != 3 {
    // NXDOMAIN (3) is not an error, just means domain doesn't exist
    return Ok(Vec::new());
  }

  let qd_count = u16::from_be_bytes([data[4], data[5]]) as usize;
  let an_count = u16::from_be_bytes([data[6], data[7]]) as usize;

  let mut pos = 12;
  for _ in 0..qd_count {
    pos = skip_name(&data, pos)? + 4;
  }

  let mut answers = Vec::with_capacity(an_count);
  for _ in 0..an_count {
    let (answer, new_pos) = parse_rr(&data, pos)?;
    answers.push(answer);
    pos = new_pos;
  }

  Ok(answers)
}

fn parse_rr(data: &Bytes, pos: usize) -> Result<(Answer, usize)> {
  let (name, mut pos) = read_name(data, pos)?;

  if pos + 10 > data.len() {
    return Err(Error::IncompleteData);
  }

  let mut slice = &data[pos..];
  let type_id = slice.get_u16();
  slice.advance(2);
  let ttl = slice.get_u32();
  let rd_len = slice.get_u16() as usize;
  pos += 10;

  if pos + rd_len > data.len() {
    return Err(Error::IncompleteData);
  }

  let rdata = data.slice(pos..pos + rd_len);
  let val = parse_rdata(data, type_id, &rdata)?;

  Ok((
    Answer {
      name,
      type_id,
      ttl,
      val,
    },
    pos + rd_len,
  ))
}

fn parse_rdata(full: &Bytes, type_id: u16, rdata: &Bytes) -> Result<String> {
  Ok(match type_id {
    1 if rdata.len() == 4 => Ipv4Addr::new(rdata[0], rdata[1], rdata[2], rdata[3]).to_string(),
    28 if rdata.len() == 16 => {
      let arr: [u8; 16] = rdata[..16].try_into().map_err(|_| Error::IncompleteData)?;
      Ipv6Addr::from(arr).to_string()
    }
    2 | 5 | 12 => read_name_at(full, rdata, 0)?.0,
    15 if rdata.len() >= 3 => {
      let pref = u16::from_be_bytes([rdata[0], rdata[1]]);
      format!("{pref} {}", read_name_at(full, rdata, 2)?.0)
    }
    16 => parse_txt(rdata),
    33 if rdata.len() >= 7 => {
      let mut r = &rdata[..];
      let (pri, wt, port) = (r.get_u16(), r.get_u16(), r.get_u16());
      format!("{pri} {wt} {port} {}", read_name_at(full, rdata, 6)?.0)
    }
    _ => hex::encode(rdata),
  })
}

fn parse_txt(rdata: &Bytes) -> String {
  let mut result = String::new();
  let mut i = 0;
  while i < rdata.len() {
    let len = rdata[i] as usize;
    i += 1;
    if i + len > rdata.len() {
      break;
    }
    if let Ok(s) = str::from_utf8(&rdata[i..i + len]) {
      if !result.is_empty() {
        result.push(' ');
      }
      result.push_str(s);
    }
    i += len;
  }
  result
}

fn read_name(data: &Bytes, pos: usize) -> Result<(String, usize)> {
  let mut name = String::with_capacity(64);
  let mut cur = pos;
  let mut jumped = false;
  let mut end_pos = pos;

  loop {
    if cur >= data.len() {
      return Err(Error::NameOutOfBounds);
    }

    let len = data[cur] as usize;

    if len == 0 {
      if !jumped {
        end_pos = cur + 1;
      }
      break;
    }

    if len & 0xC0 == 0xC0 {
      if cur + 1 >= data.len() {
        return Err(Error::PointerOutOfBounds);
      }
      let offset = ((len & 0x3F) << 8) | data[cur + 1] as usize;
      if !jumped {
        end_pos = cur + 2;
      }
      cur = offset;
      jumped = true;
      continue;
    }

    cur += 1;
    if cur + len > data.len() {
      return Err(Error::NameOutOfBounds);
    }

    if !name.is_empty() {
      name.push('.');
    }
    if let Ok(label) = str::from_utf8(&data[cur..cur + len]) {
      name.push_str(label);
    }
    cur += len;
  }

  Ok((name, end_pos))
}

fn read_name_at(full: &Bytes, rdata: &Bytes, offset: usize) -> Result<(String, usize)> {
  let full_ptr = full.as_ptr() as usize;
  let rdata_ptr = rdata.as_ptr() as usize;
  debug_assert!(rdata_ptr >= full_ptr && rdata_ptr <= full_ptr + full.len());
  read_name(full, rdata_ptr - full_ptr + offset)
}

fn skip_name(data: &Bytes, mut pos: usize) -> Result<usize> {
  loop {
    if pos >= data.len() {
      return Err(Error::NameOutOfBounds);
    }
    let len = data[pos] as usize;
    if len == 0 {
      return Ok(pos + 1);
    }
    if len & 0xC0 == 0xC0 {
      return Ok(pos + 2);
    }
    pos += 1 + len;
  }
}

/// 构建 DNS 查询消息 / Build DNS query message
///
/// - `id`: 消息 ID (DoQ 要求为 0，DoT 可随机)
/// - `domain`: 查询域名
/// - `qtype`: 查询类型
pub fn build(id: u16, domain: &str, qtype: u16) -> Bytes {
  let mut buf = BytesMut::with_capacity(64);

  // Header: ID, RD=1, QDCOUNT=1, ARCOUNT=1 (EDNS)
  buf.put_u16(id);
  buf.extend_from_slice(&[1, 0, 0, 1, 0, 0, 0, 0, 0, 1]);

  for label in domain.split('.') {
    buf.put_u8(label.len() as u8);
    buf.extend_from_slice(label.as_bytes());
  }
  buf.put_u8(0);
  buf.put_u16(qtype);
  buf.extend_from_slice(&[0, 1]); // QCLASS=IN

  // EDNS OPT: NAME=root, TYPE=41, UDP=4096, TTL=0, RDLEN=0
  buf.extend_from_slice(&[0, 0, 41, 16, 0, 0, 0, 0, 0, 0, 0]);

  buf.freeze()
}
