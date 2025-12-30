//! High-performance ts-based ID generator / 高性能时间戳ID生成器
//!
//! # ID Format / ID格式
//!
//! ```text
//! | 44 bits ts | 20 bits n |
//! |-------------------|------------------|
//! | seconds since epoch | micros within second |
//! ```
//!
//! # Features / 特性
//!
//! - Monotonic increasing IDs / 单调递增ID
//! - ~1M IDs per second / 每秒约100万个ID
//! - Clock backward tolerance / 时钟回拨容错
//! - Restart collision avoidance / 重启冲突避免

use coarsetime::Clock;

/// High-performance ID generator / 高性能ID生成器
pub struct Ider {
  ts: u64,
  n: u32,
}

// Constants for ID generation / ID生成常量
const POS_BITS: u32 = 20;
const POS_MAX: u32 = (1 << POS_BITS) - 1; // 0xFFFFF (~1M)
const MICROS_PER_SEC: u64 = 1_000_000;

impl Ider {
  /// Create new generator with microsecond-based initialization / 创建新生成器，基于微秒初始化
  ///
  /// Uses microseconds within second as initial n to avoid collision after restart
  /// 使用秒内微秒作为初始位置，避免重启后冲突
  #[inline(always)]
  pub fn new() -> Self {
    let now = Clock::now_since_epoch();
    let micros = now.as_micros() % MICROS_PER_SEC;
    Self {
      ts: now.as_secs(),
      n: micros as u32,
    }
  }

  /// Initialize generator to ensure it's ahead of last_id / 初始化生成器确保领先于last_id
  ///
  /// # Arguments / 参数
  /// * `last_id` - The last generated ID to avoid collision / 最后生成的ID，避免冲突
  ///
  /// # Safety / 安全性
  /// Must call after recovery from persistent storage to prevent ID collision
  /// 从持久化存储恢复后必须调用，防止ID碰撞
  pub fn init(&mut self, last_id: u64) {
    let last_ts = last_id >> POS_BITS;
    let last_n = (last_id & POS_MAX as u64) as u32;

    if last_ts > self.ts {
      self.ts = last_ts;
      self.n = last_n + 1;
    } else if last_ts == self.ts && last_n >= self.n {
      self.n = last_n + 1;
    }

    if self.n > POS_MAX {
      self.ts += 1;
      self.n = 0;
    }
  }

  /// Generate next unique ID / 生成下一个唯一ID
  ///
  /// # Returns / 返回值
  /// Monotonically increasing 64-bit ID / 单调递增的64位ID
  ///
  /// # Performance / 性能
  /// O(1) time complexity, no heap allocation / O(1)时间复杂度，无堆分配
  #[inline(always)]
  pub fn get(&mut self) -> u64 {
    let now = Clock::now_since_epoch().as_secs();

    // Handle clock backward and n overflow / 处理时钟回拨和位置溢出
    if now > self.ts {
      self.ts = now;
      self.n = 0;
    } else if self.n >= POS_MAX {
      self.ts += 1;
      self.n = 0;
    }

    // Compose ID: ts << POS_BITS | n / 组合ID：时间戳<<位数|位置
    let id = (self.ts << POS_BITS) | (self.n as u64);
    self.n += 1;
    id
  }
}

impl Iterator for Ider {
  type Item = u64;

  #[inline(always)]
  fn next(&mut self) -> Option<u64> {
    Some(self.get())
  }
}

impl Default for Ider {
  #[inline(always)]
  fn default() -> Self {
    Self::new()
  }
}
