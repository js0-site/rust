use super::{
  PcBase,
  types::{BlockMeta, CompactSegment},
};

/// Serialize Pc to bytes
/// 序列化 Pc 为字节流
pub fn dump<const B: usize>(pc: &PcBase<B>) -> Vec<u8> {
  let mut out = Vec::with_capacity(pc.size_in_bytes());
  out.extend_from_slice(&(pc.len as u64).to_le_bytes());

  // segments
  // 段信息
  out.extend_from_slice(&(pc.segments.len() as u32).to_le_bytes());
  for s in &pc.segments {
    out.extend_from_slice(&s.start_idx.to_le_bytes());
    out.extend_from_slice(&s.slope_fp.to_le_bytes());
    out.extend_from_slice(&s.intercept_fp.to_le_bytes());
  }

  // block_meta
  // 块元数据（每个块 13 字节：4+1+4+4）
  out.extend_from_slice(&(pc.block_meta.len() as u32).to_le_bytes());
  for b in &pc.block_meta {
    out.extend_from_slice(&b.bit_offset.to_le_bytes());
    out.push(b.bit_width);
    out.extend_from_slice(&b.seg_idx.to_le_bytes());
    out.extend_from_slice(&b.exception_offset.to_le_bytes());
  }

  // residuals
  // 残差数组
  out.extend_from_slice(&(pc.residuals.len() as u32).to_le_bytes());
  for r in &pc.residuals {
    out.extend_from_slice(&r.to_le_bytes());
  }

  // exceptions
  // 离群值数组
  out.extend_from_slice(&(pc.exceptions.len() as u32).to_le_bytes());
  for e in &pc.exceptions {
    out.extend_from_slice(&e.to_le_bytes());
  }

  // bitmap
  // 位图数组
  out.extend_from_slice(&(pc.bitmap.len() as u32).to_le_bytes());
  for b in &pc.bitmap {
    out.extend_from_slice(&b.to_le_bytes());
  }
  out
}

/// Deserialize Pc from bytes
/// 从字节流反序列化 Pc
pub fn load<const B: usize>(bytes: &[u8]) -> PcBase<B> {
  let mut pos = 0;

  let len = u64::from_le_bytes(bytes[pos..pos + 8].try_into().unwrap()) as usize;
  pos += 8;

  // segments
  // 反序列化段信息
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
  // 反序列化块元数据（每个块 13 字节）
  let block_count = u32::from_le_bytes(bytes[pos..pos + 4].try_into().unwrap()) as usize;
  pos += 4;
  let mut block_meta = Vec::with_capacity(block_count);

  let block_meta_bytes = block_count * 13;
  let meta_chunk = &bytes[pos..pos + block_meta_bytes];
  pos += block_meta_bytes;

  for chunk in meta_chunk.chunks_exact(13) {
    let bit_offset = u32::from_le_bytes(chunk[0..4].try_into().unwrap());
    let bit_width = chunk[4];
    let seg_idx = u32::from_le_bytes(chunk[5..9].try_into().unwrap());
    let exception_offset = u32::from_le_bytes(chunk[9..13].try_into().unwrap());
    block_meta.push(BlockMeta {
      bit_offset,
      bit_width,
      seg_idx,
      exception_offset,
    });
  }

  // residuals
  // 反序列化残差数组
  let res_count = u32::from_le_bytes(bytes[pos..pos + 4].try_into().unwrap()) as usize;
  pos += 4;
  let mut residuals = Vec::with_capacity(res_count);

  for _ in 0..res_count {
    residuals.push(u64::from_le_bytes(bytes[pos..pos + 8].try_into().unwrap()));
    pos += 8;
  }

  // exceptions
  // 反序列化离群值数组
  let ex_count = u32::from_le_bytes(bytes[pos..pos + 4].try_into().unwrap()) as usize;
  pos += 4;
  let mut exceptions = Vec::with_capacity(ex_count);
  for _ in 0..ex_count {
    exceptions.push(u64::from_le_bytes(bytes[pos..pos + 8].try_into().unwrap()));
    pos += 8;
  }

  // bitmap
  // 反序列化位图数组
  let bm_count = u32::from_le_bytes(bytes[pos..pos + 4].try_into().unwrap()) as usize;
  pos += 4;
  let mut bitmap = Vec::with_capacity(bm_count);
  for _ in 0..bm_count {
    bitmap.push(u64::from_le_bytes(bytes[pos..pos + 8].try_into().unwrap()));
    pos += 8;
  }

  PcBase {
    segments,
    block_meta,
    residuals,
    exceptions,
    bitmap,
    len,
  }
}
