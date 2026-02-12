use super::{PcBase, types::BlockMeta};

/// Serialize Pc to bytes
/// 序列化 Pc 为字节流
pub fn dump<const B: usize>(pc: &PcBase<B>) -> Vec<u8> {
  let mut out = Vec::with_capacity(pc.size_in_bytes());
  out.extend_from_slice(&(pc.len as u64).to_le_bytes());

  // Helper macro for vector serialization
  macro_rules! serialize_vec {
    ($vec:expr, $serializer:expr) => {
      out.extend_from_slice(&($vec.len() as u32).to_le_bytes());
      for item in &$vec {
        $serializer(item);
      }
    };
  }

  serialize_vec!(pc.block_meta, |b: &BlockMeta| {
    out.extend_from_slice(&b.bit_offset.to_le_bytes());
    out.push(b.bit_width);
    out.push(b.flags);
    out.extend_from_slice(&b.exception_offset.to_le_bytes());
    // New fields
    out.extend_from_slice(&b.slope_fp.to_le_bytes());
    out.extend_from_slice(&b.intercept_fp.to_le_bytes());
  });

  serialize_vec!(pc.residuals, |r: &u64| out
    .extend_from_slice(&r.to_le_bytes()));
  serialize_vec!(pc.exceptions, |e: &u64| out
    .extend_from_slice(&e.to_le_bytes()));
  serialize_vec!(pc.bitmap, |b: &u64| out.extend_from_slice(&b.to_le_bytes()));

  out
}

/// Deserialize Pc from bytes.
/// 从字节流反序列化 Pc。
pub fn load<const B: usize>(bytes: &[u8]) -> crate::error::Result<PcBase<B>> {
  let mut pos = 0;

  if bytes.len() < 8 {
    return Err(crate::error::PgmError::InvalidData(
      "Data too short for length header".into(),
    ));
  }
  let len = u64::from_le_bytes(bytes[pos..pos + 8].try_into().unwrap()) as usize;
  pos += 8;

  macro_rules! check_len {
    ($needed:expr) => {
      if pos + $needed > bytes.len() {
        return Err(crate::error::PgmError::InvalidData(format!(
          "Unexpected EOF at pos {}, needed {}",
          pos, $needed
        )));
      }
    };
  }

  macro_rules! read_slice {
    ($len:expr) => {{
      check_len!($len);
      let slice = &bytes[pos..pos + $len];
      pos += $len;
      slice
    }};
  }

  macro_rules! read_u32 {
    () => {{ u32::from_le_bytes(read_slice!(4).try_into().unwrap()) }};
  }

  macro_rules! read_u64 {
    () => {{ u64::from_le_bytes(read_slice!(8).try_into().unwrap()) }};
  }

  macro_rules! deserialize_vec {
    ($deserializer:expr) => {{
      let count = read_u32!() as usize;
      let mut vec = Vec::with_capacity(count);
      for _ in 0..count {
        vec.push($deserializer()?);
      }
      vec
    }};
  }

  let block_meta = deserialize_vec!(|| -> crate::error::Result<BlockMeta> {
    let bit_offset = read_u32!();
    check_len!(2); // bit_width + flags
    let bit_width = bytes[pos];
    let flags = bytes[pos + 1];
    pos += 2;
    let exception_offset = u32::from_le_bytes(read_slice!(4).try_into().unwrap());
    let slope_fp = read_u64!();
    let intercept_fp = i64::from_le_bytes(read_slice!(8).try_into().unwrap());

    Ok(BlockMeta {
      bit_offset,
      bit_width,
      flags,
      exception_offset,
      slope_fp,
      intercept_fp,
    })
  });

  let residuals = deserialize_vec!(|| -> crate::error::Result<u64> { Ok(read_u64!()) });

  let exceptions = deserialize_vec!(|| -> crate::error::Result<u64> { Ok(read_u64!()) });

  let bitmap = deserialize_vec!(|| -> crate::error::Result<u64> { Ok(read_u64!()) });

  Ok(PcBase {
    block_meta,
    residuals,
    exceptions,
    bitmap,
    len,
  })
}
