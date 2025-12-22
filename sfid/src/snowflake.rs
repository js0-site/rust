use std::{
  marker::PhantomData,
  sync::atomic::{AtomicU64, Ordering},
};

use coarsetime::Clock;
use tracing::warn;

use crate::{Layout, DefaultLayout};

/// Clock backward threshold in seconds
/// 时钟回拨告警阈值（秒）
const CLOCK_BACKWARD_WARN_SEC: u64 = 1;

/// Default epoch: 2025-12-22 00:00:00 UTC (seconds)
/// 默认纪元：2025-12-22 00:00:00 UTC（秒）
pub const EPOCH: u64 = 1766361600;

/// Snowflake ID generator with configurable bit layout
/// 可配置位布局的雪花 ID 生成器
pub struct Snowflake<L: Layout = DefaultLayout> {
  epoch: u64,
  pid: u64,
  state: AtomicU64,
  #[cfg(feature = "auto_pid")]
  _pid_handle: Option<crate::Pid>,
  _layout: PhantomData<L>,
}

impl<L: Layout> Snowflake<L> {
  pub const fn new(epoch: u64, pid: u16) -> Self {
    Self {
      epoch,
      pid: (pid as u64) & L::PID_MASK,
      state: AtomicU64::new(0),
      #[cfg(feature = "auto_pid")]
      _pid_handle: None,
      _layout: PhantomData,
    }
  }

  /// Generate next snowflake ID
  /// 生成下一个雪花 ID
  pub fn next(&self) -> i64 {
    loop {
      let ts = self.current_sec();
      let old = self.state.load(Ordering::Acquire);
      let old_ts = old >> L::SEQ_BITS;
      let old_seq = old & L::SEQ_MASK;

      let (new_ts, new_seq) = if ts > old_ts {
        (ts, 0)
      } else if old_seq < L::SEQ_MASK {
        // Clock backward: warn if > threshold
        // 时钟回拨：超过阈值则告警
        if old_ts > ts + CLOCK_BACKWARD_WARN_SEC {
          let backward = old_ts - ts;
          warn!("Clock backward {backward}s detected");
        }
        (old_ts, old_seq + 1)
      } else {
        (old_ts + 1, 0)
      };

      let new_state = (new_ts << L::SEQ_BITS) | new_seq;
      if self
        .state
        .compare_exchange_weak(old, new_state, Ordering::Release, Ordering::Relaxed)
        .is_ok()
      {
        return (((new_ts & L::TS_MASK) << L::TS_SHIFT) | (self.pid << L::SEQ_BITS) | new_seq)
          as i64;
      }
    }
  }

  #[inline]
  fn current_sec(&self) -> u64 {
    Clock::now_since_epoch()
      .as_secs()
      .saturating_sub(self.epoch)
  }
}

#[cfg(feature = "auto_pid")]
impl<L: Layout> Snowflake<L> {
  /// Create with Redis-allocated process ID
  /// 使用 Redis 分配的进程号创建
  #[cfg_attr(docsrs, doc(cfg(feature = "auto_pid")))]
  pub async fn auto(app: impl AsRef<[u8]>, epoch: u64) -> crate::Result<Self> {
    let pid_handle = crate::allocate::<L>(app).await?;
    let mut sf = Self::new(epoch, pid_handle.id());
    sf._pid_handle = Some(pid_handle);
    Ok(sf)
  }
}
