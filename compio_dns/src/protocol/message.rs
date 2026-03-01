use std::mem::size_of;

use zerocopy::FromBytes;

use super::{super::DnsError, header::RawHeader, record::Record};

pub struct Message<'a> {
  pub header: RawHeader,
  pub answers: Vec<Record<'a>>,
}

impl<'a> Message<'a> {
  pub fn read(buf: &'a [u8]) -> Result<Self, DnsError> {
    let (header, _) = RawHeader::read_from_prefix(buf).map_err(|_| DnsError::BufferTooShort)?;

    let mut idx = size_of::<RawHeader>();

    // 跳过问题段
    for _ in 0..header.qd_count.get() {
      let name_len = super::record::skip_name(buf, idx)?;
      idx += name_len;

      if buf.len() < idx + 4 {
        return Err(DnsError::BufferTooShort);
      }
      idx += 4; // qtype + qclass
    }

    let an_count = header.an_count.get() as usize;
    let mut answers = Vec::with_capacity(an_count);
    for _ in 0..an_count {
      let (record, new_idx) = Record::read(buf, idx)?;
      answers.push(record);
      idx = new_idx;
    }

    Ok(Self { header, answers })
  }
}
