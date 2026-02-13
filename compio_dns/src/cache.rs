//! DNS 解析缓存，底层用 `scc::HashCache`。
//!
//! 通过 `dns-cache` feature 启用。使用 per-bucket LRU 淘汰（32 路关联），
//! 遵循 DNS TTL 并做 min/max 钳制。

use std::{
  net::IpAddr,
  time::{Duration, Instant},
};

use scc::HashCache;

/// 缓存容量
const CAPACITY: usize = 32768;
/// 最小 TTL（钳制下限）
const MIN_TTL: Duration = Duration::from_secs(60);
/// 最大 TTL（钳制上限）
const MAX_TTL: Duration = Duration::from_secs(86400);

#[derive(Clone)]
struct Entry {
  addrs: Vec<IpAddr>,
  expire_at: Instant,
}

/// 全局无锁 DNS 缓存
pub(crate) struct Cache {
  inner: HashCache<String, Entry>,
}

impl Cache {
  pub fn new() -> Self {
    Self {
      inner: HashCache::with_capacity(CAPACITY, CAPACITY * 2),
    }
  }

  /// 查找缓存。过期或未命中返回 None。
  pub async fn get(&self, name: &str) -> Option<Vec<IpAddr>> {
    let entry = self.inner.get_async(name).await?;
    let val = entry.get();
    if val.expire_at > Instant::now() {
      Some(val.addrs.clone())
    } else {
      drop(entry);
      // 惰性淘汰过期条目
      let _ = self.inner.remove_async(name).await;
      None
    }
  }

  /// 插入解析结果，ttl_secs 为 0 时不缓存。
  pub async fn insert(&self, name: String, addrs: Vec<IpAddr>, ttl_secs: u32) {
    if ttl_secs == 0 {
      return;
    }
    let ttl = Duration::from_secs(ttl_secs as u64).clamp(MIN_TTL, MAX_TTL);
    let entry = Entry {
      addrs,
      expire_at: Instant::now() + ttl,
    };
    let _ = self.inner.put_async(name, entry).await;
  }
}
