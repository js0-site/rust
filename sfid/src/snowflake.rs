use std::sync::atomic::{AtomicI64, AtomicU16, Ordering};

use coarsetime::Clock;

use crate::{Error, Result, machine_id};

/// Custom epoch: 2020-01-01 00:00:00 UTC
/// 自定义纪元：2020-01-01 00:00:00 UTC
const EPOCH: u64 = 1577836800000;

/// Bit allocations
/// 位分配
const SEQUENCE_BITS: u8 = 12;
const MACHINE_BITS: u8 = 10;

const MAX_SEQUENCE: u16 = (1 << SEQUENCE_BITS) - 1;

const MACHINE_SHIFT: u8 = SEQUENCE_BITS;
const TIMESTAMP_SHIFT: u8 = SEQUENCE_BITS + MACHINE_BITS;

/// Max clock backwards tolerance in ms (100ms)
/// 最大时钟回拨容忍值（100毫秒）
const MAX_BACKWARDS_MS: u64 = 100;

/// Snowflake ID generator
/// 雪花 ID 生成器
pub struct Snowflake {
  last_ts: AtomicI64,
  sequence: AtomicU16,
}

impl Snowflake {
  pub const fn new() -> Self {
    Self {
      last_ts: AtomicI64::new(-1),
      sequence: AtomicU16::new(0),
    }
  }

  /// Generate next snowflake ID
  /// 生成下一个雪花 ID
  pub fn next(&self) -> Result<i64> {
    let machine = machine_id() as i64;
    let mut ts = Self::current_ms();

    loop {
      let last = self.last_ts.load(Ordering::Acquire);

      if ts < last {
        let diff = (last - ts) as u64;
        if diff > MAX_BACKWARDS_MS {
          // Clock moved backwards too much, reject
          // 时钟回拨过大，拒绝
          return Err(Error::ClockMovedBackwards(diff));
        }
        // Small backwards, wait it out
        // 小幅回拨，等待
        std::thread::sleep(std::time::Duration::from_millis(diff));
        ts = Self::current_ms();
        if ts < last {
          return Err(Error::ClockMovedBackwards((last - ts) as u64));
        }
      }

      if ts == last {
        // Same millisecond, increment sequence
        // 同一毫秒，递增序列号
        let seq = self.sequence.fetch_add(1, Ordering::AcqRel);
        if seq <= MAX_SEQUENCE {
          return Ok(Self::compose(ts, machine, seq as i64));
        }

        // Sequence overflow, wait for next millisecond
        // 序列号溢出，等待下一毫秒
        while Self::current_ms() == ts {
          std::hint::spin_loop();
        }
        ts = Self::current_ms();
      }

      // New millisecond, try to update timestamp
      // 新毫秒，尝试更新时间戳
      if self
        .last_ts
        .compare_exchange_weak(last, ts, Ordering::AcqRel, Ordering::Relaxed)
        .is_ok()
      {
        self.sequence.store(1, Ordering::Release);
        return Ok(Self::compose(ts, machine, 0));
      }
    }
  }

  #[inline]
  fn current_ms() -> i64 {
    (Clock::now_since_epoch().as_millis() - EPOCH) as i64
  }

  #[inline]
  fn compose(ts: i64, machine: i64, seq: i64) -> i64 {
    (ts << TIMESTAMP_SHIFT) | (machine << MACHINE_SHIFT) | seq
  }
}

impl Default for Snowflake {
  fn default() -> Self {
    Self::new()
  }
}
