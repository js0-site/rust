use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use zerocopy::{
  FromBytes, Immutable, KnownLayout,
  network_endian::{U16, U32},
};

use super::super::DnsError;

/// DNS 记录类型
const TYPE_A: u16 = 1;
const TYPE_AAAA: u16 = 28;
/// RecordFields 固定长度
const FIELDS_LEN: usize = size_of::<RecordFields>();

/// 记录固定字段（10 字节，零拷贝）
#[derive(FromBytes, KnownLayout, Immutable, Debug, Clone, Copy)]
#[repr(C)]
struct RecordFields {
  rtype: U16,
  rclass: U16,
  ttl: U32,
  rdlength: U16,
}

pub struct Record<'a> {
  pub rtype: u16,
  pub ttl: u32,
  pub rdata: &'a [u8],
}

impl<'a> Record<'a> {
  pub(super) fn read(buf: &'a [u8], offset: usize) -> Result<(Self, usize), DnsError> {
    let name_len = skip_name(buf, offset)?;
    let mut idx = offset + name_len;

    let fields_buf = buf
      .get(idx..idx + FIELDS_LEN)
      .ok_or(DnsError::BufferTooShort)?;
    let fields = RecordFields::read_from_bytes(fields_buf).map_err(|_| DnsError::BufferTooShort)?;
    idx += FIELDS_LEN;

    let rdlength = fields.rdlength.get() as usize;
    let rdata = buf
      .get(idx..idx + rdlength)
      .ok_or(DnsError::BufferTooShort)?;
    idx += rdlength;

    Ok((
      Self {
        rtype: fields.rtype.get(),
        ttl: fields.ttl.get(),
        rdata,
      },
      idx,
    ))
  }

  pub fn to_ip(&self) -> Option<IpAddr> {
    match self.rtype {
      TYPE_A => {
        let &bytes = <[u8; 4]>::ref_from_bytes(self.rdata).ok()?;
        Some(IpAddr::V4(Ipv4Addr::from(bytes)))
      }
      TYPE_AAAA => {
        let &bytes = <[u8; 16]>::ref_from_bytes(self.rdata).ok()?;
        Some(IpAddr::V6(Ipv6Addr::from(bytes)))
      }
      _ => None,
    }
  }
}

pub(super) fn skip_name(buf: &[u8], mut idx: usize) -> Result<usize, DnsError> {
  let start = idx;
  let mut jumped = false;
  let mut end = 0usize;
  // 限制跳跃次数，防止循环
  let mut jumps = 0;

  while jumps < 255 {
    // 安全检查：确保至少能读一个字节
    let len = *buf.get(idx).ok_or(DnsError::BufferTooShort)?;

    if len == 0 {
      if !jumped {
        end = idx + 1;
      }
      return Ok(end - start);
    }

    // 压缩指针 (11xxxxxx)
    if (len & 0xC0) == 0xC0 {
      if !jumped {
        end = idx + 2;
      }

      let b2 = *buf.get(idx + 1).ok_or(DnsError::BufferTooShort)?;
      let ptr = ((len as usize & 0x3F) << 8) | b2 as usize;

      // 指针只能向后指（虽然 RFC 没有强制，但通常如此，且为了避免循环）
      // 或者我们只是简单地更新 idx
      if ptr >= buf.len() {
        return Err(DnsError::InvalidPointerTarget);
      }

      idx = ptr;
      jumped = true;
      jumps += 1;
      continue;
    }

    // 普通标签 (00xxxxxx)
    let label_len = len as usize;
    idx += 1;

    // 检查缓冲区边界
    if idx + label_len > buf.len() {
      return Err(DnsError::BufferTooShort);
    }

    idx += label_len;
  }

  Err(DnsError::CompressionLoop)
}
