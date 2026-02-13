#![cfg_attr(docsrs, feature(doc_cfg))]

use std::{
  cell::UnsafeCell,
  ptr,
  sync::atomic::{AtomicBool, AtomicPtr, AtomicU8, AtomicUsize, Ordering},
  time::Duration,
};

use compio::time;
use defer_lite::defer;
pub use iter::CpuLoadIter;
use sysinfo::{CpuRefreshKind, RefreshKind, System};

mod iter;

/// Initial delay before first CPU sampling (ms)
/// 首次 CPU 采样前的初始延迟（毫秒）
const INIT_DELAY_MS: u64 = 100;

/// Default sampling interval (1 second)
/// 默认采样间隔（1 秒）
const DEFAULT_INTERVAL: Duration = Duration::from_secs(1);

/// Inner data shared between CpuLoad and background task
/// CpuLoad 和后台任务共享的内部数据
struct Inner {
  /// Global CPU load (0-100)
  /// 全局 CPU 负载 (0-100)
  global: AtomicU8,
  /// Core loads (0-100)
  /// 各核心负载 (0-100)
  cores: Box<[AtomicU8]>,
  /// Core indices sorted by load (ascending), protected by sorting flag
  /// 按负载升序排列的核心索引，由 sorting 标志保护
  rank: Box<[UnsafeCell<usize>]>,
  /// Round-robin cursor
  /// 轮询游标
  cursor: AtomicUsize,
  /// Sorting in progress flag
  /// 排序进行中标志
  sorting: AtomicBool,
  /// Stop signal for background task
  /// 后台任务停止信号
  stop: AtomicBool,
}

// SAFETY: rank is protected by sorting flag, only one thread sorts at a time
// rank 由 sorting 标志保护，同一时间只有一个线程排序
unsafe impl Sync for Inner {}

/// CPU Load Monitor
/// CPU 负载监控器
pub struct CpuLoad {
  ptr: AtomicPtr<Inner>,
}

impl CpuLoad {
  /// Sample CPU metrics and update
  /// 采样 CPU 指标并更新
  fn sample(sys: &mut System, inner: &Inner) {
    sys.refresh_cpu_all();

    // Update global load
    // 更新全局负载
    let g = sys.global_cpu_usage().clamp(0.0, 100.0) as u8;
    inner.global.store(g, Ordering::Relaxed);

    // Update core loads
    // 更新各核心负载
    let cpus = sys.cpus();
    for (i, cpu) in cpus.iter().enumerate() {
      let usage = cpu.cpu_usage().clamp(0.0, 100.0) as u8;
      // SAFETY: cores.len() == cpus.len(), set at init
      unsafe { inner.cores.get_unchecked(i) }.store(usage, Ordering::Relaxed);
    }
  }

  /// Get inner ref, returns None if stopped
  /// 获取内部引用，已停止则返回 None
  #[inline]
  fn inner(&self) -> Option<&Inner> {
    let p = self.ptr.load(Ordering::Acquire);
    if p.is_null() {
      None
    } else {
      // SAFETY: ptr is valid until Drop sets it to null
      Some(unsafe { &*p })
    }
  }

  /// Create monitor with default 1s interval
  /// 使用默认 1 秒间隔创建监控器
  #[inline]
  pub fn new() -> Self {
    Self::init(DEFAULT_INTERVAL)
  }

  /// Create monitor and start background sampling task
  /// 创建监控器并启动后台采样任务
  pub fn init(interval: Duration) -> Self {
    let mut sys =
      System::new_with_specifics(RefreshKind::nothing().with_cpu(CpuRefreshKind::everything()));
    sys.refresh_cpu_all();
    let n = sys.cpus().len().max(1);

    let cores: Box<[AtomicU8]> = (0..n).map(|_| AtomicU8::new(0)).collect();
    let rank: Box<[UnsafeCell<usize>]> = (0..n).map(UnsafeCell::new).collect();

    let inner = Box::new(Inner {
      global: AtomicU8::new(0),
      cores,
      rank,
      cursor: AtomicUsize::new(0),
      sorting: AtomicBool::new(false),
      stop: AtomicBool::new(false),
    });

    let ptr = Box::into_raw(inner);
    let inst = Self {
      ptr: AtomicPtr::new(ptr),
    };

    // Spawn background task with raw pointer
    // 用裸指针启动后台任务
    compio::runtime::spawn(async move {
      // Ensure memory is freed when task exits
      // 确保任务退出时释放内存
      defer! {
        // SAFETY: we own ptr, task is the sole owner after stop
        unsafe { drop(Box::from_raw(ptr)) };
      }

      time::sleep(Duration::from_millis(INIT_DELAY_MS)).await;

      loop {
        // SAFETY: ptr valid until stop signal
        let inner = unsafe { &*ptr };
        if inner.stop.load(Ordering::Acquire) {
          break;
        }
        Self::sample(&mut sys, inner);
        time::sleep(interval).await;
      }
    })
    .detach();

    inst
  }

  /// Get the index of the idlest CPU core (Round-Robin with periodic re-sort)
  /// 获取最空闲的 CPU 核心索引（轮询 + 周期性重排序）
  pub fn idlest(&self) -> usize {
    let Some(inner) = self.inner() else {
      return 0;
    };

    let n = inner.rank.len();
    if n == 0 {
      return 0;
    }

    let cur = inner.cursor.fetch_add(1, Ordering::Relaxed);

    // Trigger re-sort when cursor >= n
    // cursor >= n 时触发重排序
    if cur >= n
      && inner
        .sorting
        .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
        .is_ok()
    {
      // SAFETY: sorting flag acquired, exclusive access to rank
      // 已获取 sorting 标志，独占访问 rank
      let rank_slice =
        unsafe { std::slice::from_raw_parts_mut(inner.rank.as_ptr() as *mut usize, n) };

      // SAFETY: i always < n = cores.len()
      rank_slice
        .sort_unstable_by_key(|&i| unsafe { inner.cores.get_unchecked(i) }.load(Ordering::Relaxed));

      inner.cursor.store(1, Ordering::Relaxed);
      inner.sorting.store(false, Ordering::Release);

      // SAFETY: n > 0, rank[0] exists
      return rank_slice[0];
    }

    // Wait if sorting in progress
    // 如果正在排序则等待
    while inner.sorting.load(Ordering::Acquire) {
      std::hint::spin_loop();
    }

    // SAFETY: cur % n < n, not sorting
    unsafe { *inner.rank.get_unchecked(cur % n).get() }
  }

  /// Get the current global CPU load (0-100)
  /// 获取当前全局 CPU 负载 (0-100)
  #[inline]
  pub fn global(&self) -> u8 {
    self.inner().map_or(0, |i| i.global.load(Ordering::Relaxed))
  }

  /// Get the load of a specific core (0-100)
  /// 获取指定核心的负载 (0-100)
  #[inline]
  pub fn core(&self, idx: usize) -> Option<u8> {
    self
      .inner()
      .and_then(|i| i.cores.get(idx).map(|v| v.load(Ordering::Relaxed)))
  }

  /// Get the number of CPU cores
  /// 获取 CPU 核心数
  #[inline]
  pub fn len(&self) -> usize {
    self.inner().map_or(0, |i| i.cores.len())
  }

  /// Check if no cores (always false in practice)
  /// 检查是否无核心（实际上总是 false）
  #[inline]
  pub fn is_empty(&self) -> bool {
    self.len() == 0
  }

  /// Get cores slice for iteration
  /// 获取核心切片用于迭代
  #[inline]
  fn cores(&self) -> Option<&[AtomicU8]> {
    self.inner().map(|i| &*i.cores)
  }
}

impl Default for CpuLoad {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

impl Drop for CpuLoad {
  fn drop(&mut self) {
    let ptr = self.ptr.swap(ptr::null_mut(), Ordering::AcqRel);
    if !ptr.is_null() {
      // Signal stop, memory freed by background task via defer
      // 发送停止信号，内存由后台任务通过 defer 释放
      // SAFETY: ptr was valid
      unsafe {
        (*ptr).stop.store(true, Ordering::Release);
      }
    }
  }
}

impl<'a> IntoIterator for &'a CpuLoad {
  type Item = u8;
  type IntoIter = CpuLoadIter<'a>;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    CpuLoadIter::new(self.cores())
  }
}

#[static_init::dynamic]
pub static CPU_LOAD: CpuLoad = CpuLoad::new();
