//! Scalable cuckoo filter that grows automatically.
//! 可自动扩展的布谷鸟过滤器

use std::{
  borrow::Borrow,
  hash::{Hash, Hasher},
  marker::PhantomData,
};

#[cfg(feature = "gxhash")]
pub use gxhash::GxHasher;

use crate::base::Base;

#[cfg(feature = "museair")]
/// MuseAir hasher wrapper implementing `Default`.
/// 实现 `Default` 的 MuseAir 哈希器包装类型
#[derive(Clone, Debug)]
pub struct MuseAirHasher(pub museair::bfast::Hasher);

#[cfg(feature = "museair")]
impl MuseAirHasher {
  /// Create a new MuseAirHasher with a seed.
  /// 使用种子创建新的 MuseAirHasher
  #[inline]
  pub const fn with_seed(seed: u64) -> Self {
    Self(museair::bfast::Hasher::with_seed(seed))
  }
}

#[cfg(feature = "museair")]
impl Default for MuseAirHasher {
  #[inline]
  fn default() -> Self {
    Self::with_seed(0)
  }
}

#[cfg(feature = "museair")]
impl Hasher for MuseAirHasher {
  #[inline]
  fn finish(&self) -> u64 {
    self.0.finish()
  }

  #[inline]
  fn write(&mut self, bytes: &[u8]) {
    self.0.write(bytes);
  }
}

#[cfg(feature = "museair")]
impl core::ops::Deref for MuseAirHasher {
  type Target = museair::bfast::Hasher;

  #[inline]
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

#[cfg(feature = "museair")]
impl core::ops::DerefMut for MuseAirHasher {
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

/// Default hasher type.
/// 默认哈希器类型
#[cfg(feature = "gxhash")]
pub type DefaultHasher = gxhash::GxHasher;

/// Default hasher type.
/// 默认哈希器类型
#[cfg(all(feature = "museair", not(feature = "gxhash")))]
pub type DefaultHasher = MuseAirHasher;

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

impl<H: Default> Default for CuckooFilterBuilder<H> {
  fn default() -> Self {
    CuckooFilterBuilder {
      capacity: 100_000,
      fpp: 0.001,
      entries: 4,
      max_kicks: 512,
      hasher: H::default(),
    }
  }
}

impl CuckooFilterBuilder<DefaultHasher> {
  /// Create new builder with defaults.
  /// 使用默认值创建新构建器
  pub fn new() -> Self {
    Self::default()
  }
}

impl<H: Hasher + Clone> CuckooFilterBuilder<H> {
  /// Set initial capacity hint.
  /// 设置初始容量提示
  #[must_use]
  pub fn initial_capacity(mut self, hint: usize) -> Self {
    self.capacity = hint.max(1);
    self
  }

  /// Set false positive probability.
  /// 设置假阳性概率
  ///
  /// Probability must be in (0, 1]. Invalid values (including NaN) are clamped.
  /// 概率必须在 (0, 1] 范围内。无效值（含 NaN）会被钳制。
  #[must_use]
  pub fn false_positive_probability(mut self, p: f64) -> Self {
    debug_assert!(0.0 < p && p <= 1.0, "FPP must be in (0, 1]");
    // max/min chain also handles NaN (NaN.max(x) returns x)
    // max/min 链亦可处理 NaN（NaN.max(x) 返回 x）
    self.fpp = p.max(f64::MIN_POSITIVE).min(1.0);
    self
  }

  /// Set entries per bucket.
  /// 设置每桶条目数
  #[must_use]
  pub fn entries_per_bucket(mut self, n: usize) -> Self {
    self.entries = n.max(1);
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
  /// Create new filter with capacity hint and default false positive probability (0.001).
  /// 使用容量提示和默认假阳性概率 (0.001) 创建新过滤器
  pub fn with_capacity(capacity_hint: usize) -> Self {
    Self::new(capacity_hint, 0.001)
  }

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
  /// Compute item hash with internal hasher.
  /// 使用内部哈希器计算元素哈希
  #[inline(always)]
  fn hash_item<U: Hash + ?Sized>(&self, item: &U) -> u64 {
    crate::hash(&self.hasher, item)
  }

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
    self.filters.iter().all(|f| f.is_empty())
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

  /// Returns approximate memory usage in bytes.
  /// 返回近似内存占用字节数
  #[inline]
  pub fn bytes(&self) -> usize {
    self.bits().div_ceil(8) as usize
  }

  /// Returns the number of internal sub-filter layers (cascade depth).
  /// 返回内部子过滤器层数（级联深度）
  #[inline]
  pub fn subfilter_count(&self) -> usize {
    self.filters.len()
  }

  /// Returns the current load factor (len / capacity).
  /// 返回当前装载率 (len / capacity)
  #[inline]
  pub fn load_factor(&self) -> f64 {
    let cap = self.capacity();
    if cap == 0 {
      0.0
    } else {
      self.len() as f64 / cap as f64
    }
  }

  /// Check if item may exist.
  /// 检查元素是否可能存在
  #[inline]
  pub fn contains<U>(&self, item: &U) -> bool
  where
    T: Borrow<U>,
    U: Hash + ?Sized,
  {
    self.contains_hash(self.hash_item(item))
  }

  /// Check if item may exist directly by pre-computed hash.
  /// 直接通过预先计算的哈希值检查元素是否可能存在
  #[inline]
  pub fn contains_hash(&self, hash: u64) -> bool {
    // Reverse: newest filter more likely to contain item
    // 逆序：最新的过滤器更可能包含元素
    match self.filters.as_slice() {
      [single] => single.contains(&self.hasher, hash),
      filters => filters.iter().rev().any(|f| f.contains(&self.hasher, hash)),
    }
  }

  /// Check existence for a batch of items, writing results into the provided slice.
  /// 批量检查多个元素是否存在，将结果写入提供的切片中（零堆分配）。
  #[inline]
  pub fn contains_batch_into<U>(&self, items: &[&U], results: &mut [bool])
  where
    T: Borrow<U>,
    U: Hash + ?Sized,
  {
    assert_eq!(items.len(), results.len());
    for (res, &item) in results.iter_mut().zip(items) {
      *res = self.contains_hash(self.hash_item(item));
    }
  }

  /// Check existence for a batch of items.
  /// 批量检查多个元素是否存在
  #[inline]
  pub fn contains_batch<U>(&self, items: &[&U]) -> Vec<bool>
  where
    T: Borrow<U>,
    U: Hash + ?Sized,
  {
    let mut results = vec![false; items.len()];
    self.contains_batch_into(items, &mut results);
    results
  }

  /// Check existence for a batch of pre-computed hashes, writing results into the provided slice.
  /// 批量检查多个预先计算的哈希值是否存在，将结果写入提供的切片中（零堆分配）。
  #[inline]
  pub fn contains_hash_batch_into(&self, hashes: &[u64], results: &mut [bool]) {
    assert_eq!(hashes.len(), results.len());
    match self.filters.as_slice() {
      [single] => {
        for (res, &hash) in results.iter_mut().zip(hashes) {
          *res = single.contains(&self.hasher, hash);
        }
      }
      filters => {
        for (res, &hash) in results.iter_mut().zip(hashes) {
          *res = filters.iter().rev().any(|f| f.contains(&self.hasher, hash));
        }
      }
    }
  }

  /// Check existence for a batch of pre-computed hashes.
  /// 批量检查多个预先计算的哈希值是否存在
  #[inline]
  pub fn contains_hash_batch(&self, hashes: &[u64]) -> Vec<bool> {
    let mut results = vec![false; hashes.len()];
    self.contains_hash_batch_into(hashes, &mut results);
    results
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
    self.add_hash(self.hash_item(item));
  }

  /// Insert pre-computed hash without checking existence (UNSAFE for duplicates).
  /// 插入预先计算的哈希值但不检查是否存在（重复插入不安全）
  #[inline]
  pub fn add_hash(&mut self, hash: u64) {
    if self.filters.is_empty() {
      self.grow();
    }
    let last = self.filters.last_mut().unwrap();
    last.insert(&self.hasher, hash);
    if last.is_nearly_full() {
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
    self.add_hash_if_not_exist(self.hash_item(item))
  }

  /// Add pre-computed hash if not already present (safe insertion).
  /// 如果预计算哈希值不存在则添加（安全插入）
  ///
  /// Returns true if hash was already present.
  /// 如果哈希值已存在返回 true
  #[inline]
  pub fn add_hash_if_not_exist(&mut self, hash: u64) -> bool {
    if self.contains_hash(hash) {
      true
    } else {
      self.add_hash(hash);
      false
    }
  }

  /// Clear all items from the filter, resetting back to the initial single level.
  /// 清空过滤器中的所有元素，重置回初始单层状态
  #[inline]
  pub fn clear(&mut self) {
    self.filters.clear();
    self.grow();
  }

  /// Shrink filter capacity, pruning trailing empty cascade layers.
  /// 收缩过滤器容量，修剪尾部空闲的级联子过滤器层
  #[inline]
  pub fn shrink_to_fit(&mut self) {
    while self.filters.len() > 1 && self.filters.last().is_some_and(|f| f.is_empty()) {
      self.filters.pop();
    }
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
    self.remove_hash(self.hash_item(item))
  }

  /// Remove pre-computed hash from filter.
  /// 从过滤器移除预先计算的哈希值
  ///
  /// Returns true if removed.
  /// 如果移除成功返回 true
  #[inline]
  pub fn remove_hash(&mut self, hash: u64) -> bool {
    match self.filters.as_mut_slice() {
      [single] => single.remove(&self.hasher, hash),
      filters => filters
        .iter_mut()
        .rev()
        .any(|f| f.remove(&self.hasher, hash)),
    }
  }

  fn grow(&mut self) {
    let shift = (self.filters.len() as u32).min(usize::BITS - 1);
    let cap = self.capacity.saturating_mul(1usize << shift);
    let prob = self.fpp / 2f64.powi((self.filters.len() as i32 + 1).min(1023));
    let fp_bits = ((2.0 * self.entries as f64) / prob).log2().ceil() as usize;
    // Cap fingerprint size to prevent overflow (max 56 bits)
    // 限制指纹大小以防止溢出（最大 56 位）
    let fp_bits = fp_bits.clamp(1, 56);
    let filter = Base::new(fp_bits, self.entries, cap, self.max_kicks);
    self.filters.push(filter);
  }
}

impl<T: Hash + ?Sized, H: Hasher + Clone + Default> Default for CuckooFilter<T, H> {
  fn default() -> Self {
    CuckooFilterBuilder::default().finish()
  }
}

impl<'a, T: Hash + ?Sized, U: ?Sized + Hash, H: Hasher + Clone + Default> FromIterator<&'a U>
  for CuckooFilter<T, H>
where
  T: Borrow<U>,
{
  fn from_iter<I: IntoIterator<Item = &'a U>>(iter: I) -> Self {
    let iter = iter.into_iter();
    // Use size hint to avoid over-allocating the default 100k capacity
    // 利用大小提示避免按默认 10 万容量过度分配
    let cap = iter.size_hint().0.max(1);
    let mut filter = CuckooFilterBuilder::default()
      .initial_capacity(cap)
      .finish();
    filter.extend(iter);
    filter
  }
}

impl<'a, T: Hash + ?Sized, U: ?Sized + Hash, H: Hasher + Clone> Extend<&'a U> for CuckooFilter<T, H>
where
  T: Borrow<U>,
{
  fn extend<I: IntoIterator<Item = &'a U>>(&mut self, iter: I) {
    for item in iter {
      self.add_if_not_exist(item);
    }
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

  #[test]
  fn new_api_features() {
    let mut filter: CuckooFilter<str> = CuckooFilter::default();
    assert_eq!(filter.subfilter_count(), 1);
    assert_eq!(filter.len(), 0);
    assert!(filter.bytes() > 0);
    assert_eq!(filter.load_factor(), 0.0);

    // Test contains_batch and contains_batch_into
    let items = ["apple", "banana", "cherry"];
    let batch_res = filter.contains_batch(&items);
    assert_eq!(batch_res, vec![false, false, false]);

    let mut buf = [false; 3];
    filter.contains_batch_into(&items, &mut buf);
    assert_eq!(buf, [false, false, false]);

    // Test Extend
    filter.extend(items);
    assert_eq!(filter.len(), 3);
    assert!(filter.load_factor() > 0.0);
    let batch_res2 = filter.contains_batch(&items);
    assert_eq!(batch_res2, vec![true, true, true]);

    filter.contains_batch_into(&items, &mut buf);
    assert_eq!(buf, [true, true, true]);

    // Test FromIterator
    let collected: CuckooFilter<str> = items.into_iter().collect();
    assert_eq!(collected.len(), 3);
    assert!(collected.contains("apple"));

    // Test pre-computed hash operations
    let test_hash: u64 = 0x1234_5678_9abc_def0;
    assert!(!filter.contains_hash(test_hash));
    assert!(!filter.add_hash_if_not_exist(test_hash));
    assert!(filter.contains_hash(test_hash));
    assert!(filter.add_hash_if_not_exist(test_hash));
    assert!(filter.remove_hash(test_hash));
    assert!(!filter.contains_hash(test_hash));

    // Test clear
    filter.clear();
    assert_eq!(filter.len(), 0);
    assert!(!filter.contains("apple"));
    assert_eq!(filter.subfilter_count(), 1);
  }

  #[test]
  fn test_max_kicks_zero() {
    // With max_kicks = 0, collisions immediately fall back to exceptional storage
    // max_kicks = 0 时碰撞应立即安全存入异常区，不产生未定义状态
    let mut filter: CuckooFilter<u64> = CuckooFilterBuilder::new()
      .initial_capacity(100)
      .false_positive_probability(0.00001)
      .max_kicks(0)
      .finish();

    for i in 0..50 {
      filter.add_if_not_exist(&i);
    }
    for i in 0..50 {
      assert!(
        filter.contains(&i),
        "item {i} must be contained with max_kicks=0"
      );
    }
    for i in 0..50 {
      assert!(filter.remove(&i));
      assert!(!filter.contains(&i));
    }
    assert_eq!(filter.len(), 0);
  }

  #[test]
  fn test_hash_batch_operations() {
    let mut filter: CuckooFilter<u64> = CuckooFilter::default();
    let hashes = [0x1111u64, 0x2222, 0x3333, 0x4444];

    assert_eq!(filter.contains_hash_batch(&hashes), vec![false; 4]);

    for &h in &hashes[..2] {
      filter.add_hash_if_not_exist(h);
    }

    assert_eq!(
      filter.contains_hash_batch(&hashes),
      vec![true, true, false, false]
    );

    let mut buf = [false; 4];
    filter.contains_hash_batch_into(&hashes, &mut buf);
    assert_eq!(buf, [true, true, false, false]);
  }

  #[test]
  fn test_shrink_to_fit_prunes_empty_filters() {
    let mut filter: CuckooFilter<u64> = CuckooFilterBuilder::new()
      .initial_capacity(64)
      .false_positive_probability(0.00001)
      .finish();

    for i in 0..2000 {
      filter.add_if_not_exist(&i);
    }
    assert!(filter.subfilter_count() > 1);

    // Remove almost all items except 10
    for i in 10..2000 {
      assert!(filter.remove(&i));
    }
    assert_eq!(filter.len(), 10);

    filter.shrink_to_fit();
    assert_eq!(filter.subfilter_count(), 1);
    for i in 0..10 {
      assert!(filter.contains(&i));
    }
  }

  #[test]
  fn test_with_capacity() {
    let filter = CuckooFilter::<str>::with_capacity(500);
    assert_eq!(filter.subfilter_count(), 1);
    assert_eq!(filter.false_positive_probability(), 0.001);
  }

  #[test]
  fn test_from_iter_sizes_from_hint() {
    // FromIterator should size from the iterator, not the 100k default
    // FromIterator 应按迭代器规模分配，而非默认 10 万容量
    let collected: CuckooFilter<str> = ["a", "b", "c"].into_iter().collect();
    assert_eq!(collected.len(), 3);
    assert!(
      collected.capacity() <= 16,
      "capacity {} too large for 3 items",
      collected.capacity()
    );
    for s in ["a", "b", "c"] {
      assert!(collected.contains(s));
    }
  }
}
