use crate::{DefaultLayout, Layout};

/// Parsed snowflake ID components
/// 解析后的雪花ID组件
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParsedId {
  /// Timestamp offset from epoch (seconds)
  /// 相对纪元的时间戳偏移（秒）
  pub ts: u64,
  /// Process ID
  /// 进程号
  pub pid: u16,
  /// Sequence number
  /// 序列号
  pub seq: u32,
}

/// Parse snowflake ID with default layout
/// 使用默认布局解析雪花ID
#[inline]
pub fn parse(id: u64) -> ParsedId {
  parse_with::<DefaultLayout>(id)
}

/// Parse snowflake ID with custom layout
/// 使用自定义布局解析雪花ID
pub fn parse_with<L: Layout>(id: u64) -> ParsedId {
  ParsedId {
    ts: id >> L::TS_SHIFT,
    pid: ((id >> L::SEQ_BITS) & L::PID_MASK) as u16,
    seq: (id & L::SEQ_MASK) as u32,
  }
}
