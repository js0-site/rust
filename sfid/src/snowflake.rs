use std::sync::atomic::{AtomicU64, Ordering};

use coarsetime::Clock;
use tracing::warn;

use crate::bits::{PID_MASK, SEQ_BITS, SEQ_MASK, TS_MASK, TS_SHIFT};

/// Clock backward threshold in ms (1 second)
/// 时钟回拨告警阈值（1秒）
const CLOCK_BACKWARD_WARN_MS: u64 = 1000;

/// Default epoch: 2025-12-22 00:00:00 UTC
/// 默认纪元：2025-12-22 00:00:00 UTC
pub const EPOCH: u64 = 1766361600000;

/// Snowflake ID generator
/// 雪花 ID 生成器
pub struct Snowflake {
  epoch: u64,
  pid: u64,
  /// Packed state: high bits = timestamp, low 12 bits = sequence
  /// 打包状态：高位=时间戳，低12位=序列号
  state: AtomicU64,
  /// Hold Pid to stop heartbeat on drop
  /// 持有 Pid 以便 drop 时停止心跳
  #[cfg(feature = "auto_pid")]
  _pid_handle: Option<crate::Pid>,
}

impl Snowflake {
  pub const fn new(epoch: u64, pid: u16) -> Self {
    Self {
      epoch,
      // Mask to 10 bits to prevent overflow
      // 掩码为10位防止溢出
      pid: (pid as u64) & PID_MASK,
      state: AtomicU64::new(0),
      #[cfg(feature = "auto_pid")]
      _pid_handle: None,
    }
  }

  /// Generate next snowflake ID
  /// 生成下一个雪花 ID
  pub fn next(&self) -> i64 {
    loop {
      let ts = self.current_ms();
      let old = self.state.load(Ordering::Acquire);
      let old_ts = old >> SEQ_BITS;
      let old_seq = old & SEQ_MASK;

      let (new_ts, new_seq) = if ts > old_ts {
        (ts, 0)
      } else if old_seq < SEQ_MASK {
        // Same ts or clock backwards: borrow sequence
        // 同一毫秒或时钟回拨：借用序列号
        let backward = old_ts - ts;
        if backward > CLOCK_BACKWARD_WARN_MS {
          warn!("Clock backward {backward}ms detected");
        }
        (old_ts, old_seq + 1)
      } else {
        // Sequence exhausted, advance timestamp
        // 序列号耗尽，时间戳+1
        (old_ts + 1, 0)
      };

      let new_state = (new_ts << SEQ_BITS) | new_seq;
      if self
        .state
        .compare_exchange_weak(old, new_state, Ordering::Release, Ordering::Relaxed)
        .is_ok()
      {
        // Mask ts to 41 bits to prevent overflow into sign bit
        // 掩码时间戳为41位，防止溢出到符号位
        return (((new_ts & TS_MASK) << TS_SHIFT) | (self.pid << SEQ_BITS) | new_seq) as i64;
      }
    }
  }

  #[inline]
  fn current_ms(&self) -> u64 {
    Clock::now_since_epoch()
      .as_millis()
      .saturating_sub(self.epoch)
  }
}

impl Snowflake {
  /// Create with Redis-allocated process ID
  /// 使用 Redis 分配的进程号创建
  #[cfg(feature = "auto_pid")]
  #[cfg_attr(docsrs, doc(cfg(feature = "auto_pid")))]
  pub async fn auto(app: impl AsRef<[u8]>, epoch: u64) -> crate::Result<Self> {
    let pid_handle = crate::allocate(app).await?;
    let mut sf = Self::new(epoch, pid_handle.id());
    sf._pid_handle = Some(pid_handle);
    Ok(sf)
  }
}
