use std::{
  marker::PhantomData,
  sync::atomic::{AtomicU64, Ordering},
};

use coarsetime::Clock;
use log::warn;

use crate::{CLOCK_BACKWARD_WARN_SEC, DefaultLayout, Layout};

/// Snowflake ID generator with configurable bit layout
/// 可配置位布局的雪花 ID 生成器
pub struct SfId<L: Layout = DefaultLayout> {
  epoch: u64,
  pid: u64,
  state: AtomicU64,
  #[cfg(feature = "auto_pid")]
  _pid_handle: Option<crate::Pid>,
  _layout: PhantomData<L>,
}

impl<L: Layout> SfId<L> {
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

  #[cfg(feature = "auto_pid")]
  pub(crate) fn with_pid(epoch: u64, pid_handle: crate::Pid) -> Self {
    Self {
      epoch,
      pid: (pid_handle.id() as u64) & L::PID_MASK,
      state: AtomicU64::new(0),
      _pid_handle: Some(pid_handle),
      _layout: PhantomData,
    }
  }

  /// Generate snowflake ID
  /// 生成雪花 ID
  pub fn get(&self) -> u64 {
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
        return ((new_ts & L::TS_MASK) << L::TS_SHIFT) | (self.pid << L::SEQ_BITS) | new_seq;
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
