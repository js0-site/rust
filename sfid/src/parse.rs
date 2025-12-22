use crate::bits::{PID_MASK, SEQ_BITS, SEQ_MASK, TS_SHIFT};

/// Parsed snowflake ID components
/// 解析后的雪花ID组件
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParsedId {
  /// Timestamp offset from epoch (ms)
  /// 相对纪元的时间戳偏移（毫秒）
  pub ts: u64,
  /// Process/machine ID
  /// 进程/机器号
  pub pid: u16,
  /// Sequence number
  /// 序列号
  pub seq: u16,
}

/// Parse snowflake ID to components
/// 解析雪花ID为组件
pub fn parse(id: i64) -> ParsedId {
  let id = id as u64;
  ParsedId {
    ts: id >> TS_SHIFT,
    pid: ((id >> SEQ_BITS) & PID_MASK) as u16,
    seq: (id & SEQ_MASK) as u16,
  }
}
