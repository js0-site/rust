/// Bit layout configuration trait
/// 位布局配置 trait
pub trait Layout {
  const SEQ_BITS: u32;
  const PID_BITS: u32;
  const TS_BITS: u32;

  const SEQ_MASK: u64 = (1 << Self::SEQ_BITS) - 1;
  const PID_MASK: u64 = (1 << Self::PID_BITS) - 1;
  const TS_MASK: u64 = (1 << Self::TS_BITS) - 1;
  const TS_SHIFT: u32 = Self::SEQ_BITS + Self::PID_BITS;
  const MAX_PID: u32 = 1 << Self::PID_BITS;
}

/// Default layout: 35-bit timestamp (seconds), 10-bit process ID, 19-bit sequence
/// 默认布局：35位时间戳（秒）、10位进程号、19位序列号
pub struct DefaultLayout;

impl Layout for DefaultLayout {
  /// 35 bits = ~1088 years
  /// 35位 ≈ 1088年
  const TS_BITS: u32 = 35;
  /// 10 bits = 1024 processes
  /// 10位 = 1024进程
  const PID_BITS: u32 = 10;
  /// 19 bits = 524288/sec
  /// 19位 = 524288/秒
  const SEQ_BITS: u32 = 19;
}
