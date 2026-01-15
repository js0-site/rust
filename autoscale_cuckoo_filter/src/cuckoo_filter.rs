//! Scalable cuckoo filter that grows automatically.
//! 可自动扩展的布谷鸟过滤器

use std::{
  borrow::Borrow,
  hash::{Hash, Hasher},
  marker::PhantomData,
};

use gxhash::GxHasher;

use crate::base::Base;

/// Default hasher type.
/// 默认哈希器类型
pub type DefaultHasher = GxHasher;

/// Builder for CuckooFilter.
/// CuckooFilter 构建器
#[derive(Debug)]
pub struct CuckooFilterBuilder<H = DefaultHasher> {
  capacity: usize,
  fpp: f64,
  entries: usize,
  max_kicks: usize,
  hasher: H,
}

impl CuckooFilterBuilder<DefaultHasher> {
  /// Create new builder with defaults.
  /// 使用默认值创建新构建器
  pub fn new() -> Self {
    CuckooFilterBuilder {
      capacity: 100_000,
      fpp: 0.001,
      entries: 4,
      max_kicks: 512,
      hasher: GxHasher::default(),
    }
  }
}

impl<H: Hasher + Clone> CuckooFilterBuilder<H> {
  /// Set initial capacity hint.
  /// 设置初始容量提示
  #[must_use]
  pub fn initial_capacity(mut self, hint: usize) -> Self {
    self.capacity = hint;
    self
  }

  /// Set false positive probability.
  /// 设置假阳性概率
  ///
  /// Probability must be in (0, 1]. Invalid values are clamped.
  /// 概率必须在 (0, 1] 范围内。无效值会被钳制。
  #[must_use]
  pub fn false_positive_probability(mut self, p: f64) -> Self {
    debug_assert!(0.0 < p && p <= 1.0, "FPP must be in (0, 1]");
    self.fpp = p.clamp(f64::MIN_POSITIVE, 1.0);
    self
  }

  /// Set entries per bucket.
  /// 设置每桶条目数
  #[must_use]
  pub fn entries_per_bucket(mut self, n: usize) -> Self {
    self.entries = n;
    self
  }

  /// Set max kicks before grow.
  /// 设置扩展前最大踢出次数
  #[must_use]
  pub fn max_kicks(mut self, kicks: usize) -> Self {
    self.max_kicks = kicks;
    self
  }

  /// Set custom hasher.
  /// 设置自定义哈希器
  pub fn hasher<T: Hasher>(self, hasher: T) -> CuckooFilterBuilder<T> {
    CuckooFilterBuilder {
      capacity: self.capacity,
      fpp: self.fpp,
      entries: self.entries,
      max_kicks: self.max_kicks,
      hasher,
    }
  }

  /// Build the filter.
  /// 构建过滤器
  pub fn finish<T: Hash + ?Sized>(self) -> CuckooFilter<T, H> {
    let mut filter = CuckooFilter {
      hasher: self.hasher,
      capacity: self.capacity,
      fpp: self.fpp,
      entries: self.entries,
      max_kicks: self.max_kicks,
      filters: Vec::new(),
      _item: PhantomData,
    };
    filter.grow();
    filter
  }
}

impl Default for CuckooFilterBuilder {
  fn default() -> Self {
    Self::new()
  }
}

/// Scalable Cuckoo Filter that grows automatically.
/// 可自动扩展的布谷鸟过滤器
///
/// # Examples
///
/// ```rust
/// use autoscale_cuckoo_filter::CuckooFilter;
///
/// let mut filter = CuckooFilter::<str>::new(1000, 0.001);
/// filter.add_if_not_exist("hello");
/// assert!(filter.contains("hello"));
/// ```
///
/// For types with inner references, use wrapper types:
/// 对于包含内部引用的类型，使用包装类型：
///
/// ```rust
/// #[derive(Hash)]
/// struct InnerTuple<'a>(&'a str, Option<&'a str>);
///
/// #[derive(Hash)]
/// struct MyTuple(InnerTuple<'static>);
///
/// impl<'a> std::borrow::Borrow<InnerTuple<'a>> for MyTuple {
///     fn borrow(&self) -> &InnerTuple<'a> {
///         &self.0
///     }
/// }
///
/// let mut filter = autoscale_cuckoo_filter::CuckooFilter::<MyTuple>::new(1000, 0.05);
/// let a = "hello".to_string();
/// let q = InnerTuple(&a[..], None);
/// filter.add_if_not_exist(&q);
/// ```
#[derive(Debug, bitcode::Encode, bitcode::Decode)]
pub struct CuckooFilter<T: ?Sized, H = DefaultHasher> {
  #[bitcode(skip)]
  hasher: H,
  filters: Vec<Base>,
  capacity: usize,
  fpp: f64,
  entries: usize,
  max_kicks: usize,
  #[bitcode(skip)]
  _item: PhantomData<T>,
}

impl<T: Hash + ?Sized> CuckooFilter<T> {
  /// Create new filter with capacity hint and false positive probability.
  /// 使用容量提示和假阳性概率创建新过滤器
  pub fn new(capacity_hint: usize, fpp: f64) -> Self {
    CuckooFilterBuilder::new()
      .initial_capacity(capacity_hint)
      .false_positive_probability(fpp)
      .finish()
  }
}

impl<T: Hash + ?Sized, H: Hasher + Clone> CuckooFilter<T, H> {
  /// Returns approximate item count.
  /// 返回近似元素数量
  #[inline]
  pub fn len(&self) -> usize {
    self.filters.iter().map(|f| f.len()).sum()
  }

  /// Returns true if empty.
  /// 如果为空返回 true
  #[inline]
  pub fn is_empty(&self) -> bool {
    self.len() == 0
  }

  /// Returns total capacity.
  /// 返回总容量
  #[inline]
  pub fn capacity(&self) -> usize {
    self.filters.iter().map(|f| f.capacity()).sum()
  }

  /// Returns bits used.
  /// 返回使用的位数
  #[inline]
  pub fn bits(&self) -> u64 {
    self.filters.iter().map(|f| f.bits()).sum()
  }

  /// Returns false positive probability.
  /// 返回假阳性概率
  #[inline]
  pub fn false_positive_probability(&self) -> f64 {
    self.fpp
  }

  /// Returns entries per bucket.
  /// 返回每桶条目数
  #[inline]
  pub fn entries_per_bucket(&self) -> usize {
    self.entries
  }

  /// Returns max kicks.
  /// 返回最大踢出次数
  #[inline]
  pub fn max_kicks(&self) -> usize {
    self.max_kicks
  }

  /// Check if item may exist.
  /// 检查元素是否可能存在
  #[inline]
  pub fn contains<U>(&self, item: &U) -> bool
  where
    T: Borrow<U>,
    U: Hash + ?Sized,
  {
    let hash = crate::hash(&self.hasher, item);
    self.contains_hash(hash)
  }

  #[inline]
  fn contains_hash(&self, hash: u64) -> bool {
    // Reverse: newest filter more likely to contain item
    // 逆序：最新的过滤器更可能包含元素
    self
      .filters
      .iter()
      .rev()
      .any(|f| f.contains(&self.hasher, hash))
  }

  /// Insert item without checking existence (UNSAFE for duplicates).
  /// 插入元素但不检查是否存在（重复插入不安全）
  ///
  /// Filter grows automatically when full.
  /// 满时自动扩展
  ///
  /// **WARNING**: This method does NOT check if item already exists.
  /// Repeatedly inserting the same item will create duplicate entries and cause memory bloat.
  /// Use `add_if_not_exist()` instead for safe insertion.
  ///
  /// **警告**：此方法不检查元素是否已存在。
  /// 重复插入相同元素会创建重复条目导致内存膨胀。
  /// 请使用 `add_if_not_exist()` 进行安全插入。
  #[inline]
  pub fn add<U>(&mut self, item: &U)
  where
    T: Borrow<U>,
    U: Hash + ?Sized,
  {
    let hash = crate::hash(&self.hasher, item);
    self.insert_hash(hash);
  }

  #[inline]
  fn insert_hash(&mut self, hash: u64) {
    let last = self.filters.len() - 1;
    self.filters[last].insert(&self.hasher, hash);
    if self.filters[last].is_nearly_full() {
      self.grow();
    }
  }

  /// Add item if not already present (safe insertion).
  /// 如果元素不存在则添加（安全插入）
  ///
  /// More efficient than `contains` + `insert` (single hash).
  /// 比 `contains` + `insert` 更高效（单次哈希）
  ///
  /// Returns true if item was already present.
  /// 如果元素已存在返回 true
  #[inline]
  pub fn add_if_not_exist<U>(&mut self, item: &U) -> bool
  where
    T: Borrow<U>,
    U: Hash + ?Sized,
  {
    let hash = crate::hash(&self.hasher, item);
    if self.contains_hash(hash) {
      true
    } else {
      self.insert_hash(hash);
      false
    }
  }

  /// Shrink filter capacity.
  /// 收缩过滤器容量
  #[inline]
  pub fn shrink_to_fit(&mut self) {
    for f in &mut self.filters {
      f.shrink_to_fit(&self.hasher);
    }
  }

  /// Remove item from filter.
  /// 从过滤器移除元素
  ///
  /// Returns true if removed.
  /// 如果移除成功返回 true
  #[inline]
  pub fn remove<U>(&mut self, item: &U) -> bool
  where
    T: Borrow<U>,
    U: Hash + ?Sized,
  {
    let hash = crate::hash(&self.hasher, item);
    for filter in &mut self.filters {
      if filter.remove(&self.hasher, hash) {
        return true;
      }
    }
    false
  }

  fn grow(&mut self) {
    let cap = self.capacity * 2usize.pow(self.filters.len() as u32);
    let prob = self.fpp / 2f64.powi(self.filters.len() as i32 + 1);
    let fp_bits = ((1.0 / prob).log2() + ((2 * self.entries) as f64).log2()).ceil() as usize;
    // Cap fingerprint size to prevent overflow (max 56 bits)
    // 限制指纹大小以防止溢出（最大 56 位）
    let fp_bits = fp_bits.min(56);
    let filter = Base::new(fp_bits, self.entries, cap, self.max_kicks);
    self.filters.push(filter);
  }
}

impl<T: Hash + ?Sized, H: Hasher + Clone> Clone for CuckooFilter<T, H> {
  fn clone(&self) -> Self {
    Self {
      hasher: self.hasher.clone(),
      filters: self.filters.clone(),
      capacity: self.capacity,
      fpp: self.fpp,
      entries: self.entries,
      max_kicks: self.max_kicks,
      _item: self._item,
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn basic_ops() {
    let mut filter = CuckooFilter::<str>::new(1000, 0.001);
    assert!(filter.is_empty());
    assert!(filter.bits() > 0);

    assert!(!filter.contains("foo"));
    filter.add_if_not_exist("foo");
    assert!(filter.contains("foo"));
  }

  #[test]
  fn clone_works() {
    let mut filter: CuckooFilter<String> = CuckooFilter::new(1000, 0.001);
    filter.add_if_not_exist(&"foo".to_owned());
    let cloned = filter.clone();
    assert!(filter.contains(&"foo".to_string()));
    assert!(cloned.contains(&"foo".to_string()));

    let mut filter: CuckooFilter<str> = CuckooFilter::new(1000, 0.001);
    filter.add_if_not_exist("foo");
    let cloned = filter.clone();
    assert!(filter.contains("foo"));
    assert!(cloned.contains("foo"));
  }

  #[test]
  fn add_many() {
    let mut filter: CuckooFilter<u64> = CuckooFilterBuilder::new()
      .initial_capacity(100)
      .false_positive_probability(0.00001)
      .finish();
    for i in 0..10_000 {
      assert!(!filter.contains(&i));
      filter.add_if_not_exist(&i);
      assert!(filter.contains(&i));
    }
    assert_eq!(filter.len(), 10_000);
  }

  #[test]
  fn remove_works() {
    let mut filter: CuckooFilter<usize> = CuckooFilterBuilder::new()
      .initial_capacity(100)
      .false_positive_probability(0.00001)
      .finish();

    for i in 0..10_000 {
      filter.add_if_not_exist(&i);
    }
    for i in 0..10_000 {
      assert!(filter.remove(&i));
      assert!(!filter.contains(&i));
    }
    for i in 0..10_000 {
      assert!(!filter.remove(&i));
    }
  }

  #[test]
  fn duplicate_remove() {
    let mut filter = CuckooFilter::<str>::new(1000, 0.001);
    filter.add("foo");
    filter.add("foo");
    assert!(filter.contains("foo"));

    filter.remove("foo");
    assert!(filter.contains("foo"));

    filter.remove("foo");
    assert!(!filter.contains("foo"));
  }

  #[test]
  fn shrink_works() {
    let mut filter = CuckooFilter::<i32>::new(1000, 0.001);
    for i in 0..100 {
      filter.add_if_not_exist(&i);
    }
    assert_eq!(filter.capacity(), 1024);
    assert!(filter.bits() > 0);

    filter.shrink_to_fit();
    for i in 0..100 {
      assert!(filter.contains(&i));
    }
    assert_eq!(filter.capacity(), 128);
    assert!(filter.bits() > 0);
  }

  #[test]
  fn info_params() {
    let mut filter = CuckooFilter::<u64>::new(10, 0.001);

    assert_eq!(filter.max_kicks(), 512);
    assert_eq!(filter.entries_per_bucket(), 4);
    assert_eq!(filter.false_positive_probability(), 0.001);
    assert!(filter.bits() > 0);
    assert!(filter.capacity() >= 16);

    for i in 0..100 {
      filter.add_if_not_exist(&i);
    }

    assert!(filter.bits() > 0);
    assert!(filter.capacity() >= 100);
  }

  #[test]
  fn serde_works() {
    let mut filter = CuckooFilter::<usize>::new(1000, 0.001);
    for i in 0..100 {
      filter.add_if_not_exist(&i);
    }
    filter.shrink_to_fit();
    let serialized = bitcode::encode(&filter);
    let deserialized: CuckooFilter<usize> = bitcode::decode(&serialized).unwrap();
    for i in 0..100 {
      assert!(filter.contains(&i));
      assert!(deserialized.contains(&i));
    }
  }
}
