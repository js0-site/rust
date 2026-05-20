#![cfg_attr(docsrs, feature(doc_cfg))]

use std::{
  fmt::{self, Debug, Display},
  iter::Map,
  net::{Ipv4Addr, Ipv6Addr},
  vec::IntoIter,
};

/// IP range trait / IP 范围特征
pub trait IpRange: Copy {
  type Int: Ord + Copy;

  fn to_int(self) -> Self::Int;
  fn from_cidr(addr: Self, prefix: u8) -> Range<Self::Int>;

  /// Check if addr is in CIDR / 检查 addr 是否在 CIDR 内
  fn in_cidr(net: Self, prefix: u8, addr: Self) -> bool {
    Self::from_cidr(net, prefix).contains(&addr.to_int())
  }
}

impl IpRange for Ipv4Addr {
  type Int = u32;

  fn to_int(self) -> u32 {
    u32::from(self)
  }

  fn from_cidr(addr: Self, prefix: u8) -> Range<u32> {
    let ip = u32::from(addr);
    if prefix == 0 {
      return Range {
        start: 0,
        end: u32::MAX,
      };
    }
    if prefix >= 32 {
      return Range { start: ip, end: ip };
    }
    let mask = !0u32 << (32 - prefix);
    Range {
      start: ip & mask,
      end: (ip & mask) | !mask,
    }
  }
}

impl IpRange for Ipv6Addr {
  type Int = u128;

  fn to_int(self) -> u128 {
    u128::from(self)
  }

  fn from_cidr(addr: Self, prefix: u8) -> Range<u128> {
    let ip = u128::from(addr);
    if prefix == 0 {
      return Range {
        start: 0,
        end: u128::MAX,
      };
    }
    if prefix >= 128 {
      return Range { start: ip, end: ip };
    }
    let mask = !0u128 << (128 - prefix);
    Range {
      start: ip & mask,
      end: (ip & mask) | !mask,
    }
  }
}

/// Integer range / 整数范围
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Range<T> {
  pub start: T,
  pub end: T,
}

impl<T: Ord> Range<T> {
  pub fn contains(&self, val: &T) -> bool {
    val >= &self.start && val <= &self.end
  }
}

impl<T: Debug> Debug for Range<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{:?}..={:?}", self.start, self.end)
  }
}

/// IPv4 range / IPv4 范围
pub type Ip4Range = Range<u32>;
/// IPv6 range / IPv6 范围
pub type Ip6Range = Range<u128>;

impl Display for Ip4Range {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let start = Ipv4Addr::from(self.start);
    let end = Ipv4Addr::from(self.end);
    if self.start == self.end {
      write!(f, "{start}")
    } else {
      write!(f, "{start}-{end}")
    }
  }
}

impl Display for Ip6Range {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let start = Ipv6Addr::from(self.start);
    let end = Ipv6Addr::from(self.end);
    if self.start == self.end {
      write!(f, "{start}")
    } else {
      write!(f, "{start}-{end}")
    }
  }
}

/// Sorted IP map with binary search / 排序的 IP 映射（插入时保持有序）
#[derive(Clone)]
pub struct IpMap<T: IpRange, V> {
  li: Vec<(Range<T::Int>, V)>,
}

impl<T: IpRange, V> Default for IpMap<T, V> {
  fn default() -> Self {
    Self::new()
  }
}

impl<T: IpRange, V> IpMap<T, V> {
  pub fn new() -> Self {
    Self { li: Vec::new() }
  }

  fn insert(&mut self, range: Range<T::Int>, val: V) {
    let idx = self.li.partition_point(|(r, _)| r.start < range.start);
    self.li.insert(idx, (range, val));
  }

  /// Add single IP with value / 添加单个 IP 及其值
  pub fn add(&mut self, addr: T, val: V) {
    let ip = addr.to_int();
    self.insert(Range { start: ip, end: ip }, val);
  }

  /// Add CIDR range with value / 添加 CIDR 范围及其值
  pub fn add_cidr(&mut self, addr: T, prefix: u8, val: V) {
    self.insert(T::from_cidr(addr, prefix), val);
  }

  pub fn is_empty(&self) -> bool {
    self.li.is_empty()
  }

  pub fn len(&self) -> usize {
    self.li.len()
  }

  pub fn first(&self) -> Option<&(Range<T::Int>, V)> {
    self.li.first()
  }

  pub fn iter(&self) -> impl Iterator<Item = &(Range<T::Int>, V)> {
    self.li.iter()
  }

  pub fn contains(&self, addr: T) -> bool {
    let ip = addr.to_int();
    let idx = self.li.partition_point(|(r, _)| r.start <= ip);
    idx > 0 && ip <= self.li[idx - 1].0.end
  }

  pub fn get(&self, addr: T) -> Option<&V> {
    let ip = addr.to_int();
    let idx = self.li.partition_point(|(r, _)| r.start <= ip);
    if idx > 0 && ip <= self.li[idx - 1].0.end {
      Some(&self.li[idx - 1].1)
    } else {
      None
    }
  }
}

impl<T: IpRange, V> IntoIterator for IpMap<T, V> {
  type Item = (Range<T::Int>, V);
  type IntoIter = IntoIter<(Range<T::Int>, V)>;

  fn into_iter(self) -> Self::IntoIter {
    self.li.into_iter()
  }
}

pub type Ipv4Map<V> = IpMap<Ipv4Addr, V>;
pub type Ipv6Map<V> = IpMap<Ipv6Addr, V>;

/// Sorted IP set based on IpMap / 基于 IpMap 的排序 IP 集合
#[derive(Clone)]
pub struct IpSet<T: IpRange>(IpMap<T, ()>);

impl<T: IpRange> Default for IpSet<T> {
  fn default() -> Self {
    Self::new()
  }
}

impl<T: IpRange> IpSet<T> {
  pub fn new() -> Self {
    Self(IpMap::new())
  }

  /// Add single IP / 添加单个 IP
  pub fn add(&mut self, addr: T) {
    self.0.add(addr, ());
  }

  /// Add CIDR range / 添加 CIDR 范围
  pub fn add_cidr(&mut self, addr: T, prefix: u8) {
    self.0.add_cidr(addr, prefix, ());
  }

  pub fn contains(&self, addr: T) -> bool {
    self.0.contains(addr)
  }

  pub fn is_empty(&self) -> bool {
    self.0.is_empty()
  }

  pub fn len(&self) -> usize {
    self.0.len()
  }

  pub fn iter(&self) -> impl Iterator<Item = &Range<T::Int>> {
    self.0.iter().map(|(r, _)| r)
  }
}

impl<T: IpRange> IntoIterator for IpSet<T> {
  type Item = Range<T::Int>;
  type IntoIter = Map<IntoIter<(Range<T::Int>, ())>, fn((Range<T::Int>, ())) -> Range<T::Int>>;

  fn into_iter(self) -> Self::IntoIter {
    self.0.into_iter().map(|(r, _)| r)
  }
}

pub type Ipv4Set = IpSet<Ipv4Addr>;
pub type Ipv6Set = IpSet<Ipv6Addr>;

impl Display for Ipv4Set {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "[")?;
    for (i, r) in self.iter().enumerate() {
      if i > 0 {
        write!(f, " / ")?;
      }
      write!(f, "{}", Ip4Range { ..*r })?;
    }
    write!(f, "]")
  }
}

impl Debug for Ipv4Set {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    Display::fmt(self, f)
  }
}

impl Display for Ipv6Set {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "[")?;
    for (i, r) in self.iter().enumerate() {
      if i > 0 {
        write!(f, " / ")?;
      }
      write!(f, "{}", Ip6Range { ..*r })?;
    }
    write!(f, "]")
  }
}

impl Debug for Ipv6Set {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    Display::fmt(self, f)
  }
}

impl<V: Debug> Debug for Ipv4Map<V> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    Display::fmt(self, f)
  }
}

impl<V: Debug> Display for Ipv4Map<V> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "[")?;
    for (i, (r, v)) in self.iter().enumerate() {
      if i > 0 {
        write!(f, ", ")?;
      }
      write!(f, "{}: {v:?}", Ip4Range { ..*r })?;
    }
    write!(f, "]")
  }
}

impl<V: Debug> Debug for Ipv6Map<V> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    Display::fmt(self, f)
  }
}

impl<V: Debug> Display for Ipv6Map<V> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "[")?;
    for (i, (r, v)) in self.iter().enumerate() {
      if i > 0 {
        write!(f, ", ")?;
      }
      write!(f, "{}: {v:?}", Ip6Range { ..*r })?;
    }
    write!(f, "]")
  }
}
