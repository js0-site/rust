use std::net::{Ipv4Addr, Ipv6Addr};

use aok::{OK, Void};
use ip_set::{
  Ip4Range, Ip6Range, IpMap, IpRange, IpSet, Ipv4Map, Ipv4Set, Ipv6Map, Ipv6Set, Range,
};
use log::info;

#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
}

#[test]
fn test_ipv4_set() -> Void {
  let mut set = Ipv4Set::new();

  set.add(Ipv4Addr::new(192, 168, 1, 100));
  set.add_cidr(Ipv4Addr::new(10, 0, 0, 0), 24);

  info!("Ipv4Set: {set}");
  info!("Ipv4Set Debug: {set:?}");

  assert!(set.contains(Ipv4Addr::new(192, 168, 1, 100)));
  assert!(set.contains(Ipv4Addr::new(10, 0, 0, 1)));
  assert!(set.contains(Ipv4Addr::new(10, 0, 0, 255)));
  assert!(!set.contains(Ipv4Addr::new(10, 0, 1, 0)));
  assert!(!set.contains(Ipv4Addr::new(192, 168, 1, 101)));

  assert_eq!(set.len(), 2);
  assert!(!set.is_empty());

  // iter
  let ranges: Vec<_> = set.iter().collect();
  assert_eq!(ranges.len(), 2);

  // into_iter
  let ranges: Vec<_> = set.into_iter().collect();
  assert_eq!(ranges.len(), 2);

  OK
}

#[test]
fn test_ipv6_set() -> Void {
  let mut set = Ipv6Set::new();

  set.add(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1));
  set.add_cidr(Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 0), 64);

  info!("Ipv6Set: {set}");
  info!("Ipv6Set Debug: {set:?}");

  assert!(set.contains(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1)));
  assert!(set.contains(Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 1)));
  assert!(!set.contains(Ipv6Addr::new(0xfe80, 0, 0, 1, 0, 0, 0, 0)));

  OK
}

#[test]
fn test_in_cidr() -> Void {
  let net = Ipv4Addr::new(192, 168, 0, 0);
  assert!(Ipv4Addr::in_cidr(net, 16, Ipv4Addr::new(192, 168, 1, 1)));
  assert!(Ipv4Addr::in_cidr(
    net,
    16,
    Ipv4Addr::new(192, 168, 255, 255)
  ));
  assert!(!Ipv4Addr::in_cidr(net, 16, Ipv4Addr::new(192, 169, 0, 0)));

  let net6 = Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0);
  assert!(Ipv6Addr::in_cidr(
    net6,
    32,
    Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1)
  ));
  assert!(!Ipv6Addr::in_cidr(
    net6,
    32,
    Ipv6Addr::new(0x2001, 0xdb9, 0, 0, 0, 0, 0, 0)
  ));

  OK
}

#[test]
fn test_edge_cases() -> Void {
  // prefix=32: single IP
  let mut set = Ipv4Set::new();
  set.add_cidr(Ipv4Addr::new(1, 2, 3, 4), 32);
  assert!(set.contains(Ipv4Addr::new(1, 2, 3, 4)));
  assert!(!set.contains(Ipv4Addr::new(1, 2, 3, 5)));

  // prefix=0: all IPs
  assert!(Ipv4Addr::in_cidr(
    Ipv4Addr::new(0, 0, 0, 0),
    0,
    Ipv4Addr::new(255, 255, 255, 255)
  ));

  // prefix > 32: treated as single IP
  let range = Ipv4Addr::from_cidr(Ipv4Addr::new(1, 2, 3, 4), 33);
  assert_eq!(range.start, range.end);

  // prefix=128 for IPv6
  let range6 = Ipv6Addr::from_cidr(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1), 128);
  assert_eq!(range6.start, range6.end);

  // prefix=0 for IPv6
  let range6 = Ipv6Addr::from_cidr(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0), 0);
  assert_eq!(range6.start, 0);
  assert_eq!(range6.end, u128::MAX);

  // prefix > 128 for IPv6
  let range6 = Ipv6Addr::from_cidr(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1), 129);
  assert_eq!(range6.start, range6.end);

  // empty set
  let empty = Ipv4Set::new();
  assert!(empty.is_empty());
  assert_eq!(empty.len(), 0);
  assert!(!empty.contains(Ipv4Addr::new(1, 1, 1, 1)));

  OK
}

#[test]
fn test_ipv4_map() -> Void {
  let mut map: Ipv4Map<&str> = Ipv4Map::new();

  map.add(Ipv4Addr::new(192, 168, 1, 100), "single");
  map.add_cidr(Ipv4Addr::new(10, 0, 0, 0), 24, "internal");
  map.add_cidr(Ipv4Addr::new(172, 16, 0, 0), 16, "private");

  info!("Ipv4Map: {map}");
  info!("Ipv4Map Debug: {map:?}");

  assert_eq!(map.get(Ipv4Addr::new(192, 168, 1, 100)), Some(&"single"));
  assert_eq!(map.get(Ipv4Addr::new(10, 0, 0, 1)), Some(&"internal"));
  assert_eq!(map.get(Ipv4Addr::new(172, 16, 255, 255)), Some(&"private"));
  assert_eq!(map.get(Ipv4Addr::new(8, 8, 8, 8)), None);

  assert_eq!(map.len(), 3);
  assert!(!map.is_empty());

  // first
  let first = map.first();
  assert!(first.is_some());

  // iter
  let entries: Vec<_> = map.iter().collect();
  assert_eq!(entries.len(), 3);

  // into_iter
  let entries: Vec<_> = map.into_iter().collect();
  assert_eq!(entries.len(), 3);

  OK
}

#[test]
fn test_ipv6_map() -> Void {
  let mut map: Ipv6Map<i32> = Ipv6Map::new();

  map.add(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1), 1);
  map.add_cidr(Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 0), 64, 2);

  info!("Ipv6Map: {map}");
  info!("Ipv6Map Debug: {map:?}");

  assert_eq!(
    map.get(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1)),
    Some(&1)
  );
  assert_eq!(
    map.get(Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 100)),
    Some(&2)
  );
  assert_eq!(
    map.get(Ipv6Addr::new(0x2001, 0xdb9, 0, 0, 0, 0, 0, 0)),
    None
  );

  OK
}

#[test]
fn test_range() -> Void {
  let r = Range {
    start: 10u32,
    end: 20u32,
  };
  assert!(r.contains(&10));
  assert!(r.contains(&15));
  assert!(r.contains(&20));
  assert!(!r.contains(&9));
  assert!(!r.contains(&21));

  // Debug
  let debug = format!("{r:?}");
  assert!(debug.contains("10"));
  assert!(debug.contains("20"));

  OK
}

#[test]
fn test_ip_range_display() -> Void {
  // Ip4Range single IP
  let r = Ip4Range {
    start: 167772161,
    end: 167772161,
  }; // 10.0.0.1
  let s = format!("{r}");
  assert_eq!(s, "10.0.0.1");

  // Ip4Range range
  let r = Ip4Range {
    start: 167772160,
    end: 167772415,
  }; // 10.0.0.0-10.0.0.255
  let s = format!("{r}");
  assert!(s.contains("10.0.0.0"));
  assert!(s.contains("10.0.0.255"));

  // Ip6Range single IP
  let ip6 = u128::from(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1));
  let r = Ip6Range {
    start: ip6,
    end: ip6,
  };
  let s = format!("{r}");
  assert!(s.contains("2001:db8::1"));

  // Ip6Range range
  let start = u128::from(Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 0));
  let end = u128::from(Ipv6Addr::new(
    0xfe80, 0, 0, 0, 0xffff, 0xffff, 0xffff, 0xffff,
  ));
  let r = Ip6Range { start, end };
  let s = format!("{r}");
  assert!(s.contains("fe80::"));

  OK
}

#[test]
fn test_to_int() -> Void {
  let ip4 = Ipv4Addr::new(10, 0, 0, 1);
  assert_eq!(ip4.to_int(), 167772161);

  let ip6 = Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1);
  assert_eq!(ip6.to_int(), u128::from(ip6));

  OK
}

#[test]
fn test_clone() -> Void {
  let mut set = Ipv4Set::new();
  set.add(Ipv4Addr::new(1, 2, 3, 4));
  let cloned = set.clone();
  assert_eq!(cloned.len(), 1);
  assert!(cloned.contains(Ipv4Addr::new(1, 2, 3, 4)));

  let mut map: Ipv4Map<i32> = Ipv4Map::new();
  map.add(Ipv4Addr::new(1, 2, 3, 4), 42);
  let cloned = map.clone();
  assert_eq!(cloned.len(), 1);
  assert_eq!(cloned.get(Ipv4Addr::new(1, 2, 3, 4)), Some(&42));

  OK
}

#[test]
fn test_default() -> Void {
  let set: IpSet<Ipv4Addr> = Default::default();
  assert!(set.is_empty());

  let map: IpMap<Ipv4Addr, i32> = Default::default();
  assert!(map.is_empty());

  OK
}

#[test]
fn test_empty_map() -> Void {
  let map: Ipv4Map<i32> = Ipv4Map::new();
  assert!(map.is_empty());
  assert_eq!(map.len(), 0);
  assert_eq!(map.get(Ipv4Addr::new(1, 1, 1, 1)), None);
  assert!(map.first().is_none());

  OK
}
