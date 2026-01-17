use super::{
  Pc,
  types::{BlockMeta, CompactSegment},
};

/// Serialize Pc to bytes
/// 序列化 Pc 为字节流
pub fn dump(pc: &Pc) -> Vec<u8> {
  let mut out = Vec::with_capacity(pc.size_in_bytes());
  out.extend_from_slice(&(pc.len as u64).to_le_bytes());

  // segments
  out.extend_from_slice(&(pc.segments.len() as u32).to_le_bytes());
  for s in &pc.segments {
    out.extend_from_slice(&s.start_idx.to_le_bytes());
    out.extend_from_slice(&s.slope_fp.to_le_bytes());
    out.extend_from_slice(&s.intercept_fp.to_le_bytes());
  }

  // block_meta
  out.extend_from_slice(&(pc.block_meta.len() as u32).to_le_bytes());
  for b in &pc.block_meta {
    out.extend_from_slice(&b.bit_offset.to_le_bytes());
    out.push(b.bit_width);
  }

  // residuals
  out.extend_from_slice(&(pc.residuals.len() as u32).to_le_bytes());
  for r in &pc.residuals {
    out.extend_from_slice(&r.to_le_bytes());
  }
  out
}

/// Deserialize Pc from bytes
/// 从字节流反序列化 Pc
pub fn load(bytes: &[u8]) -> Pc {
  let mut pos = 0;

  let len = u64::from_le_bytes(bytes[pos..pos + 8].try_into().unwrap()) as usize;
  pos += 8;

  // segments
  let seg_count = u32::from_le_bytes(bytes[pos..pos + 4].try_into().unwrap()) as usize;
  pos += 4;
  let mut segments = Vec::with_capacity(seg_count);
  for _ in 0..seg_count {
    let start_idx = u32::from_le_bytes(bytes[pos..pos + 4].try_into().unwrap());
    pos += 4;
    let slope_fp = u64::from_le_bytes(bytes[pos..pos + 8].try_into().unwrap());
    pos += 8;
    let intercept_fp = i64::from_le_bytes(bytes[pos..pos + 8].try_into().unwrap());
    pos += 8;
    segments.push(CompactSegment {
      start_idx,
      slope_fp,
      intercept_fp,
    });
  }

  // block_meta
  let block_count = u32::from_le_bytes(bytes[pos..pos + 4].try_into().unwrap()) as usize;
  pos += 4;
  let mut block_meta = Vec::with_capacity(block_count);
  for _ in 0..block_count {
    let bit_offset = u32::from_le_bytes(bytes[pos..pos + 4].try_into().unwrap());
    pos += 4;
    let bit_width = bytes[pos];
    pos += 1;
    block_meta.push(BlockMeta {
      bit_offset,
      bit_width,
    });
  }

  // residuals
  let res_count = u32::from_le_bytes(bytes[pos..pos + 4].try_into().unwrap()) as usize;
  pos += 4;
  let mut residuals = Vec::with_capacity(res_count);
  for _ in 0..res_count {
    residuals.push(u64::from_le_bytes(bytes[pos..pos + 8].try_into().unwrap()));
    pos += 8;
  }

  Pc {
    segments,
    block_meta,
    residuals,
    len,
  }
}
