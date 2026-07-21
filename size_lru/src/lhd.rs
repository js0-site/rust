//! LHD (Least Hit Density) cache for variable-sized data
//!
//! Based on: Beckmann et al. "LHD: Improving Cache Hit Rate by Maximizing Hit Density" (NSDI 2018)
//!
//! Core idea: evict items with lowest `expected_hits` / size
//!
//! LHD（最低命中密度）缓存，用于变长数据
//!
//! 基于论文：Beckmann 等人的 LHD 算法
//!
//! 核心思想：淘汰 预期命中数/大小 最低的条目

use std::{borrow::Borrow, hash::Hash};

use fastrand::Rng;
use rapidhash::RapidHashMap;

use crate::{NoOnRm, OnRm, SizeLru};

// Eviction sample count
// 淘汰采样数
const SAMPLES: usize = 256;

// Age class count for hit pattern
// 命中模式年龄类别数
const AGE_CLASSES: usize = 16;

// Max age buckets (fits in u16)
// 最大年龄桶数（可存入 u16）
const MAX_AGE: usize = 4096;
const MAX_AGE_U16: u16 = MAX_AGE as u16;

// Flattened bucket count
// 扁平化桶总数
const TOTAL_BUCKETS: usize = AGE_CLASSES * MAX_AGE;

// Per-entry overhead bytes
// 每条目开销字节
const ENTRY_OVERHEAD: u32 = 96;

// EWMA decay factor
// EWMA 衰减因子
const DECAY: f32 = 0.9;

// Reconfig interval
// 重配置间隔
const RECONFIG: u64 = 1 << 15;

// Age coarsening divisor
// 年龄粗化除数
const AGE_DIVISOR: f32 = 40.96;

/// Hot metadata for eviction sampling (`SoA` layout)
/// 淘汰采样热元数据（SoA 布局）
#[derive(Clone, Copy)]
#[repr(C)]
struct Meta {
  ts: u64,
  size: u32,
  last_age: u16,
  prev_age: u16,
}

/// Cold payload
/// 冷载荷
struct Payload<K, V> {
  key: K,
  val: V,
}

/// Per-age bucket stats
/// 年龄桶统计
#[derive(Clone, Copy, Default)]
#[repr(C)]
struct Bucket {
  hits: f32,
  evicts: f32,
  density: f32,
}

/// LHD cache with random sampling eviction
/// 随机采样淘汰的 LHD 缓存
#[must_use]
pub struct Lhd<K, V, OnRm = NoOnRm> {
  // Hot/cold split for cache locality
  // 热/冷分离提升缓存局部性
  metas: Vec<Meta>,
  payloads: Vec<Payload<K, V>>,
  index: RapidHashMap<K, usize>,
  // Flattened stats buckets
  // 扁平化统计桶
  buckets: Box<[Bucket]>,
  total: usize,
  max: usize,
  ts: u64,
  shift: u32,
  last_cfg: u64,
  rng: Rng,
  on_rm: OnRm,
}

// Initialize buckets density ~ 1/age (GDSF-like)
// 初始化密度 ~ 1/age（类 GDSF）
fn init_buckets() -> Box<[Bucket]> {
  let mut buckets = vec![Bucket::default(); TOTAL_BUCKETS].into_boxed_slice();
  buckets.chunks_exact_mut(MAX_AGE).for_each(|chunk| {
    chunk.iter_mut().enumerate().for_each(|(i, b)| {
      b.density = 1.0 / (i as f32 + 1.0);
    });
  });
  buckets
}

impl<K, V> Lhd<K, V> {
  /// Create new cache with max size
  /// 创建指定最大大小的缓存
  #[inline]
  pub fn new(max: usize) -> Self {
    Self::with_on_rm(max, NoOnRm)
  }

  /// Create new cache with callback
  /// 创建带回调的缓存
  #[inline]
  pub fn with_on_rm<Rm>(max: usize, on_rm: Rm) -> Lhd<K, V, Rm> {
    Lhd {
      metas: Vec::new(),
      payloads: Vec::new(),
      index: RapidHashMap::default(),
      buckets: init_buckets(),
      total: 0,
      max: max.max(1),
      ts: 0,
      shift: 6,
      last_cfg: 0,
      rng: Rng::new(),
      on_rm,
    }
  }
}

impl<K: Hash + Eq + Clone, V> SizeLru<K, V> for Lhd<K, V> {
  type WithRm<Rm> = Lhd<K, V, Rm>;

  #[inline]
  fn with_on_rm<Rm>(max: usize, on_rm: Rm) -> Lhd<K, V, Rm> {
    Lhd::with_on_rm(max, on_rm)
  }

  #[inline]
  fn get<Q>(&mut self, key: &Q) -> Option<&V>
  where
    K: Borrow<Q>,
    Q: Hash + Eq + ?Sized,
  {
    Lhd::get(self, key)
  }

  #[inline]
  fn peek<Q>(&self, key: &Q) -> Option<&V>
  where
    K: Borrow<Q>,
    Q: Hash + Eq + ?Sized,
  {
    Lhd::peek(self, key)
  }

  #[inline]
  fn set(&mut self, key: K, val: V, size: u32) {
    Lhd::set(self, key, val, size);
  }

  #[inline]
  fn rm<Q>(&mut self, key: &Q)
  where
    K: Borrow<Q>,
    Q: Hash + Eq + ?Sized,
  {
    Lhd::rm(self, key);
  }

  #[inline]
  fn is_empty(&self) -> bool {
    self.metas.is_empty()
  }

  #[inline]
  fn len(&self) -> usize {
    self.metas.len()
  }
}

impl<K: Hash + Eq, V, F: OnRm<K, Self>> Lhd<K, V, F> {
  /// Get value and update stats
  /// 获取值并更新统计
  #[inline]
  pub fn get<Q>(&mut self, key: &Q) -> Option<&V>
  where
    K: Borrow<Q>,
    Q: Hash + Eq + ?Sized,
  {
    self.tick();
    let &idx = self.index.get(key)?;
    let ts = self.ts;
    let shift = self.shift;

    // Access hot metadata
    // 访问热元数据
    let m = unsafe { self.metas.get_unchecked_mut(idx) };
    let age = ((ts.saturating_sub(m.ts) >> shift) as usize).min(MAX_AGE - 1);
    let cid = Self::class_id(m.last_age as u32 + m.prev_age as u32);

    m.prev_age = m.last_age;
    m.last_age = age as u16;
    m.ts = ts;

    unsafe {
      self.buckets.get_unchecked_mut(cid * MAX_AGE + age).hits += 1.0;
    }

    // Access cold payload
    // 访问冷载荷
    Some(&unsafe { self.payloads.get_unchecked(idx) }.val)
  }

  /// Peek value without updating stats (for cache check)
  /// 查看值但不更新统计（用于缓存检查）
  #[inline(always)]
  pub fn peek<Q>(&self, key: &Q) -> Option<&V>
  where
    K: Borrow<Q>,
    Q: Hash + Eq + ?Sized,
  {
    let &idx = self.index.get(key)?;
    Some(&unsafe { self.payloads.get_unchecked(idx) }.val)
  }

  /// Insert with size
  /// 插入并指定大小
  #[inline]
  pub fn set(&mut self, key: K, val: V, size: u32)
  where
    K: Clone,
  {
    self.tick();
    let size = size + ENTRY_OVERHEAD;
    let sz = size as usize;

    if let Some(&idx) = self.index.get(&key) {
      let m = unsafe { self.metas.get_unchecked_mut(idx) };
      self.total = self.total.wrapping_sub(m.size as usize).wrapping_add(sz);
      m.size = size;
      m.ts = self.ts;
      unsafe { self.payloads.get_unchecked_mut(idx) }.val = val;
      return;
    }

    while self.total + sz > self.max && !self.metas.is_empty() {
      self.evict();
    }

    let idx = self.metas.len();
    self.index.insert(key.clone(), idx);

    self.metas.push(Meta {
      ts: self.ts,
      size,
      last_age: 0,
      prev_age: MAX_AGE_U16,
    });
    self.payloads.push(Payload { key, val });
    self.total += sz;
  }

  /// Remove by key
  /// 按键删除
  #[inline]
  pub fn rm<Q>(&mut self, key: &Q)
  where
    K: Borrow<Q>,
    Q: Hash + Eq + ?Sized,
  {
    if let Some(&idx) = self.index.get(key) {
      let key_ptr = unsafe { &raw const self.payloads.get_unchecked(idx).key };
      let ptr = self as *const Self;
      self.on_rm.call(unsafe { &*key_ptr }, unsafe { &*ptr });
      self.index.remove(key);
      self.rm_idx(idx);
    }
  }

  #[inline]
  fn rm_idx(&mut self, idx: usize) {
    let n = self.metas.len();
    debug_assert!(n > 0 && idx < n);
    let last = n - 1;
    if idx != last {
      self.metas.swap(idx, last);
      self.payloads.swap(idx, last);
      let moved_key = unsafe { &self.payloads.get_unchecked(idx).key };
      unsafe {
        *self.index.get_mut(moved_key).unwrap_unchecked() = idx;
      }
    }
    let m = unsafe { self.metas.pop().unwrap_unchecked() };
    self.payloads.pop();
    self.stat_evict(&m);
    self.total -= m.size as usize;
  }

  /// Get density for meta
  /// 获取元数据的密度
  #[inline(always)]
  fn density(&self, m: &Meta) -> f32 {
    let age = self.age(m.ts);
    let cid = Self::class_id(m.last_age as u32 + m.prev_age as u32);
    unsafe { self.buckets.get_unchecked(cid * MAX_AGE + age).density }
  }

  /// Evict with callback
  /// 带回调淘汰
  ///
  /// # Internal State During Callback
  /// # 回调期间的内部状态
  ///
  /// When callback fires, cache is in intermediate state:
  /// 回调触发时，缓存处于中间状态：
  /// - victim index still in `self.index`
  /// - victim 索引仍在 `self.index` 中
  /// - metas/payloads not yet modified
  /// - metas/payloads 尚未修改
  /// - `self.total` not yet decremented
  /// - `self.total` 尚未减少
  ///
  /// Calling rm/set in callback causes:
  /// 回调只能用 `peek` 获取值，`get/rm/set` 需要 `&mut self` 无法调用
  /// - set: may trigger recursive evict, corrupting victim selection
  /// - rm: swap_remove may move victim to different index, causing double-free or leak
  #[inline]
  fn evict(&mut self) {
    let n = self.metas.len();
    if n == 0 {
      return;
    }

    let samples = SAMPLES.min(n);
    let mut victim = self.rng.usize(0..n);

    // Cross-multiply for density/size comparison
    // 交叉乘法比较 density/size
    let m = unsafe { self.metas.get_unchecked(victim) };
    let mut min_d = self.density(m);
    let mut min_s = m.size;

    (1..samples).for_each(|_| {
      let idx = self.rng.usize(0..n);
      let m = unsafe { self.metas.get_unchecked(idx) };
      let d = self.density(m);
      let s = m.size;
      // Cross-multiply for density/size comparison
      // 交叉乘法比较 density/size
      // d/s < min_d/min_s
      if d * (min_s as f32) < min_d * (s as f32) {
        min_d = d;
        min_s = s;
        victim = idx;
      }
    });

    // Callback before removal or eviction
    // 删除/淘汰前回调
    let key_ptr = unsafe { &raw const self.payloads.get_unchecked(victim).key };
    let ptr = self as *const Self;
    self.on_rm.call(unsafe { &*key_ptr }, unsafe { &*ptr });

    self.index.remove(unsafe { &*key_ptr });
    self.rm_idx(victim);
  }

  #[inline]
  pub fn size(&self) -> usize {
    self.total
  }

  #[inline]
  pub fn len(&self) -> usize {
    self.metas.len()
  }

  #[inline]
  pub fn is_empty(&self) -> bool {
    self.metas.is_empty()
  }

  #[inline(always)]
  fn tick(&mut self) {
    self.ts = self.ts.wrapping_add(1);
    if self.ts.wrapping_sub(self.last_cfg) >= RECONFIG {
      self.reconfig();
    }
  }

  #[inline(always)]
  fn age(&self, ts: u64) -> usize {
    ((self.ts.saturating_sub(ts) >> self.shift) as usize).min(MAX_AGE - 1)
  }

  #[inline(always)]
  fn class_id(age: u32) -> usize {
    if age == 0 {
      return AGE_CLASSES - 1;
    }
    let lz = age.leading_zeros() as usize;
    lz.saturating_sub(19).min(AGE_CLASSES - 1)
  }

  #[inline]
  fn stat_evict(&mut self, m: &Meta) {
    let age = self.age(m.ts);
    let cid = Self::class_id(m.last_age as u32 + m.prev_age as u32);
    unsafe {
      self.buckets.get_unchecked_mut(cid * MAX_AGE + age).evicts += 1.0;
    }
  }

  #[cold]
  fn reconfig(&mut self) {
    self.last_cfg = self.ts;

    self.buckets.chunks_exact_mut(MAX_AGE).for_each(|chunk| {
      let mut events = 0.0f32;
      let mut hits_sum = 0.0f32;
      let mut life = 0.0f32;

      chunk.iter_mut().rev().for_each(|b| {
        b.hits *= DECAY;
        b.evicts *= DECAY;
        hits_sum += b.hits;
        events += b.hits + b.evicts;
        life += events;
        // Epsilon avoids div by zero
        // epsilon 避免除零
        b.density = hits_sum / (life + 1e-9);
      });
    });

    // Adapt age coarsening
    // 自适应年龄粗化
    let n = self.metas.len();
    if n > 0 {
      let opt = (n as f32 / AGE_DIVISOR) as u32;
      let opt = opt.min(1 << 20); // Cap to prevent next_power_of_two() overflow/panic
      // 限制大小以防止 next_power_of_two() 溢出/崩溃
      let s = if opt <= 1 {
        0
      } else {
        opt.next_power_of_two().trailing_zeros()
      };
      self.shift = s.min(20);
    }
  }
}
