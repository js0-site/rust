mod error;
mod r#impl;

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

pub use error::{Error, Result};
use hipstr::HipStr;
use parking_lot::{Mutex, RawMutex, lock_api::RawMutex as _};

// 预加载时长（秒）/ preload duration in seconds
pub const PRELOAD_SEC: u64 = 60;
pub const STEP_MIN: u64 = 1;
pub const STEP_MAX: u64 = 1000000;
// Redis hash key / Redis 哈希键
pub const KVID_KEY: &str = "kvid";

#[derive(Debug, Clone, Copy)]
struct Seg {
  id: u64,
  max: u64,
}

#[derive(Debug)]
struct Slow {
  next: Option<Seg>,
  step: u64,
  ts: u64,
}

impl Slow {
  const fn new() -> Self {
    Self {
      next: None,
      step: STEP_MIN,
      // 0 表示从未获取过 / 0 means never fetched
      ts: 0,
    }
  }
}

#[derive(Debug)]
struct Fast {
  id: AtomicU64,
  max: AtomicU64,
  lock: AtomicBool,
}

impl Fast {
  const fn new() -> Self {
    Self {
      id: AtomicU64::new(0),
      max: AtomicU64::new(0),
      lock: AtomicBool::new(false),
    }
  }
}

pub struct KvId {
  pub name: HipStr<'static>,
  fast: Fast,
  slow: Mutex<Slow>,
}

impl KvId {
  pub const fn const_new(name: &'static str) -> Self {
    Self {
      name: HipStr::borrowed(name),
      fast: Fast::new(),
      // RawMutex::INIT 是初始状态值，非共享实例
      // RawMutex::INIT is init state value, not shared instance
      slow: Mutex::const_new(RawMutex::INIT, Slow::new()),
    }
  }

  pub fn new(name: impl Into<HipStr<'static>>) -> Self {
    Self {
      name: name.into(),
      fast: Fast::new(),
      slow: Mutex::new(Slow::new()),
    }
  }

  // 后台填充 / background fill
  fn spawn_fill(&'static self) {
    if self.fast.lock.swap(true, Ordering::Acquire) {
      return;
    }
    tokio::spawn(async move {
      self.fill().await;
    });
  }

  async fn fill(&self) {
    let now = ts_::sec();
    let step = {
      let s = self.slow.lock();
      if s.next.is_some() {
        self.fast.lock.store(false, Ordering::Release);
        return;
      }
      Self::calc_step(&s, now)
    };
    if let Ok(seg) = self.fetch(step).await {
      let mut s = self.slow.lock();
      if s.next.is_none() {
        s.next = Some(seg);
      }
    }
    self.fast.lock.store(false, Ordering::Release);
  }

  pub async fn next(&'static self) -> Result<u64> {
    // 快速路径（完全无锁）/ fast path (lock-free)
    if let Some(id) = self.try_next() {
      // 后台预加载 / background preload
      if !self.fast.lock.load(Ordering::Relaxed) {
        self.spawn_fill();
      }
      return Ok(id);
    }

    // 慢路径 / slow path
    {
      let mut s = self.slow.lock();
      if let Some(id) = self.try_next() {
        return Ok(id);
      }
      if let Some(seg) = s.next.take() {
        self.set_seg(seg);
        // set_seg 后 try_next 必定成功 / try_next must succeed after set_seg
        let id = self.try_next().ok_or(Error::Empty)?;
        self.spawn_fill();
        return Ok(id);
      }
    }

    // 同步获取 / sync fetch
    let now = ts_::sec();
    let step = {
      let s = self.slow.lock();
      Self::calc_step(&s, now).max(s.step)
    };
    let seg = self.fetch(step).await?;
    {
      let mut s = self.slow.lock();
      self.set_seg(seg);
      s.step = step;
      s.ts = now;
    }
    let id = self.try_next().ok_or(Error::Empty)?;
    self.spawn_fill();
    Ok(id)
  }
}
