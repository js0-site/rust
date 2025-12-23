#![cfg_attr(docsrs, feature(doc_cfg))]

use std::{
  marker::PhantomData,
  ops::Deref,
  sync::atomic::{AtomicU32, Ordering},
};

/// 策略 Trait: 定义耗时与权重的转换逻辑 / Strategy trait for latency-to-weight conversion
pub trait Rank {
  /// 计算权重 / Calculate weight
  /// - `base`: 基准值 / base value
  /// - `latency_us`: 延迟(微秒) / latency in microseconds
  fn calc(base: u32, latency_us: u32) -> u32;
}

/// 默认策略: 倒数模型 / Default strategy: Inverse model
/// 逻辑: Weight = BASE / Latency
pub struct Inverse;

impl Rank for Inverse {
  fn calc(base: u32, latency: u32) -> u32 {
    base / (1 + latency)
  }
}

/// 节点: 包含数据和权重
pub struct Node<T> {
  /// 节点数据
  pub data: T,
  /// 节点权重 (原子)
  pub weight: AtomicU32,
}

impl<T> Node<T> {
  /// 创建节点
  pub fn new(data: T, weight: u32) -> Self {
    Self {
      data,
      weight: AtomicU32::new(weight),
    }
  }
}

impl<T> Deref for Node<T> {
  type Target = T;
  fn deref(&self) -> &Self::Target {
    &self.data
  }
}

/// 选中节点的句柄
/// 包含节点引用和索引
pub struct Handle<'a, T> {
  pub index: usize,
  pub node: &'a Node<T>,
}

impl<'a, T> Deref for Handle<'a, T> {
  type Target = T;
  fn deref(&self) -> &Self::Target {
    &self.node.data
  }
}

// ========================================================================
// 核心结构 (PickFast)
// ========================================================================

/// 极速加权负载均衡器 / Ultra-fast weighted load balancer
///
/// 泛型参数 / Generic parameters:
/// - `T`: 节点数据类型 / Node data type
/// - `M`: 权重模型 / Weight model (default: `Inverse`)
pub struct PickFast<T, M = Inverse> {
  /// 节点列表 / Node list
  pub li: Vec<Node<T>>,

  /// 总权重 (原子缓存) / Total weight (atomic cache)
  pub total: AtomicU32,

  /// 基准值，根据节点数自动计算 / Base value, auto-calculated from node count
  base: u32,

  _marker: PhantomData<M>,
}

unsafe impl<T: Sync, M> Sync for PickFast<T, M> {}
unsafe impl<T: Send, M> Send for PickFast<T, M> {}

impl<T, M: Rank> PickFast<T, M> {
  /// 创建选择器 / Create a new selector
  pub fn new(data: impl IntoIterator<Item = T>) -> Self {
    let li: Vec<Node<T>> = data.into_iter().map(|d| Node::new(d, 0)).collect();

    let n = li.len();
    assert!(n > 0, "PickFast: node count must be > 0");

    // 根据节点数计算 base，EMA 中 old*31 需要不溢出，所以 /32
    // Calculate base from node count, /32 to prevent old*31 overflow in EMA
    let base = u32::MAX / (n as u32) / 32;

    // 初始权重 / Initial weight
    let init_val = (base / (n as u32)).max(1);

    for node in &li {
      node.weight.store(init_val, Ordering::Relaxed);
    }

    let total = AtomicU32::new(init_val * (n as u32));

    Self {
      li,
      total,
      base,
      _marker: PhantomData,
    }
  }

  /// 节点数量
  pub fn len(&self) -> usize {
    self.li.len()
  }

  /// 是否为空
  pub fn is_empty(&self) -> bool {
    self.li.is_empty()
  }

  /// 极速挑选 (Pick)
  ///
  /// O(1) 获取总权重 -> O(N) 扫描
  pub fn pick(&self) -> Handle<'_, T> {
    let total_w = self.total.load(Ordering::Relaxed);

    if total_w == 0 {
      return Handle {
        index: 0,
        node: &self.li[0],
      };
    }

    // 随机目标
    let target = fastrand::u32(0..total_w);
    let mut sum = 0;

    // 扫描
    for (i, node) in self.li.iter().enumerate() {
      sum += node.weight.load(Ordering::Relaxed);
      if sum > target {
        return Handle { index: i, node };
      }
    }

    // 兜底 (处理并发更新时的微小窗口)
    let last = self.li.len() - 1;
    Handle {
      index: last,
      node: &self.li[last],
    }
  }

  /// 设定观测值 / Set observed value
  ///
  /// 传入观测值(如耗时)，内部自动计算权重并平滑更新
  /// Pass observed value (e.g. latency), auto-calculate and smooth-update weight
  pub fn set(&self, index: usize, val: u32) {
    if index >= self.li.len() {
      return;
    }

    let target_w = M::calc(self.base, val.max(1)).max(1);

    // CAS 更新单节点权重，32次半衰期 / CAS update node weight, 32-sample half-life
    // EMA: new = (old * 31 + target) / 32
    let _ = self.li[index]
      .weight
      .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |old| {
        Some(((old * 31 + target_w) >> 5).max(1))
      })
      .map(|prev| {
        let new_w = ((prev * 31 + target_w) >> 5).max(1);
        if new_w > prev {
          self.total.fetch_add(new_w - prev, Ordering::Relaxed);
        } else {
          self.total.fetch_sub(prev - new_w, Ordering::Relaxed);
        }
      });
  }

  /// 标记节点失败 (Failed)
  ///
  /// 将指定节点权重减半，最低为1，用于处理节点故障或性能下降
  pub fn failed(&self, index: usize) {
    if index >= self.li.len() {
      return;
    }

    // CAS 更新单节点权重，减半但不低于1 / CAS update node weight, halve but not below 1
    let _ = self.li[index]
      .weight
      .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |old| {
        Some((old >> 1).max(1))
      })
      .map(|prev| {
        // 修正总权重 / Adjust total weight
        let new_w = (prev >> 1).max(1);
        self.total.fetch_sub(prev - new_w, Ordering::Relaxed);
      });
  }

  /// 返回循环迭代器，起始位置使用加权随机选择
  ///
  /// 使用与 `pick()` 相同的加权随机算法选择起始位置
  #[cfg(feature = "iter")]
  pub fn iter(&self) -> citer::CIter<'_, Node<T>> {
    let total_w = self.total.load(Ordering::Relaxed);

    let start_pos = if total_w == 0 {
      0
    } else {
      let target = fastrand::u32(0..total_w);
      let mut sum = 0;

      let mut pos = 0;
      for (i, node) in self.li.iter().enumerate() {
        sum += node.weight.load(Ordering::Relaxed);
        if sum > target {
          pos = i;
          break;
        }
      }
      pos
    };

    citer::CIter::new(&self.li[..], start_pos)
  }
}
