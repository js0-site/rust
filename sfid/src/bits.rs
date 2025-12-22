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

/// Default layout: 36-bit timestamp (seconds), 11-bit process ID, 17-bit sequence
/// 默认布局：36位时间戳（秒）、11位进程号、17位序列号
pub struct DefaultLayout;

impl Layout for DefaultLayout {
  /// 36 bits = ~2177 years
  /// 36位 ≈ 2177年
  const TS_BITS: u32 = 36;
  /// 11 bits = 2048 processes
  /// 11位 = 2048进程
  const PID_BITS: u32 = 11;
  /// 17 bits = 131072/sec
  /// 17位 = 131072/秒
  const SEQ_BITS: u32 = 17;
}
