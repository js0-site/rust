#![cfg_attr(docsrs, feature(doc_cfg))]

mod error;
mod get_by_kvrocks;

use std::{borrow::Borrow, collections::BTreeMap, ops::Deref, sync::Arc, time::Duration};

pub use error::{Error, Result};
use expire_set::ExpireSet;
pub use get_by_kvrocks::get_by_kvrocks;
use papaya::HashMap;
use parking_lot::Mutex;
pub use ssl_trait::SslConfig;
use tokio::{spawn, time::sleep};

pub type DeadlineTs = u64;

#[static_init::dynamic]
pub static CACHE: HashMap<String, Arc<SslConfig>> = HashMap::new();

#[static_init::dynamic]
pub static EXPIRE: Mutex<BTreeMap<u64, String>> = Mutex::new(BTreeMap::new());

#[static_init::dynamic(lazy)]
pub static NOT_EXIST: ExpireSet<String> = ExpireSet::new(30);

xboot::add!({
  spawn(async move {
    loop {
      sleep(Duration::from_secs(3600)).await;
      let deadline = coarsetime::Clock::now_since_epoch().as_secs() + 4000;
      let mut btree = EXPIRE.lock();
      let keep = btree.split_off(&deadline);
      let pinned = CACHE.pin();
      for host in btree.values() {
        pinned.remove(host);
      }
      *btree = keep;
    }
  });
});

// Cert wrapper / Cert 包装器
pub struct Cert(Arc<SslConfig>);

pub async fn get(host: impl Into<String>) -> Result<Option<Cert>> {
  let host = host.into();

  // Check cache / 检查缓存
  {
    let pinned = CACHE.pin();
    if let Some(cfg) = pinned.get(&host) {
      return Ok(Some(Cert(Arc::clone(cfg))));
    }
  }

  if NOT_EXIST.contains(&host) {
    return Ok(None);
  }

  // Fetch from kvrocks / 从 kvrocks 获取
  let Some((cfg, deadline)) = get_by_kvrocks(&host).await? else {
    NOT_EXIST.insert(host);
    return Ok(None);
  };

  EXPIRE.lock().insert(deadline, host.clone());
  let cfg = Arc::new(cfg);
  CACHE.pin().insert(host, Arc::clone(&cfg));
  Ok(Some(Cert(cfg)))
}

impl Borrow<SslConfig> for Cert {
  fn borrow(&self) -> &SslConfig {
    &self.0
  }
}

impl Deref for Cert {
  type Target = SslConfig;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

#[derive(Clone)]
pub struct CertByHost;

impl ssl_trait::CertByHost for CertByHost {
  type Item = Cert;
  async fn get(&self, host: &str) -> anyhow::Result<Option<Self::Item>> {
    Ok(get(host).await?)
  }
}
