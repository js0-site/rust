pub const SEQ_BITS: u32 = 12;
pub const PID_BITS: u32 = 10;
pub const SEQ_MASK: u64 = (1 << SEQ_BITS) - 1;
pub const PID_MASK: u64 = (1 << PID_BITS) - 1;
pub const TS_SHIFT: u32 = SEQ_BITS + PID_BITS;

/// Timestamp mask (41 bits)
/// 时间戳掩码（41位）
pub const TS_MASK: u64 = (1 << 41) - 1;

/// Maximum process ID count (10 bits = 1024)
/// 进程号数量上限（10位 = 1024）
pub const MAX_PID: u32 = 1 << PID_BITS;
