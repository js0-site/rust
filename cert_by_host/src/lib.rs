#![cfg_attr(docsrs, feature(doc_cfg))]

mod error;
mod get_by_kvrocks;

use std::collections::BTreeMap;

use dashmap::{DashMap, mapref::one::Ref};
pub use error::{Error, Result};
use expire_set::ExpireSet;
pub use get_by_kvrocks::get_by_kvrocks;
use parking_lot::Mutex;
pub use ssl_trait::SslConfig;

pub type DeadlineTs = u64;

#[static_init::dynamic]
pub static CACHE: DashMap<String, SslConfig> = DashMap::new();

#[static_init::dynamic]
pub static EXPIRE: Mutex<BTreeMap<u64, String>> = Mutex::new(BTreeMap::new());

xboot::add!({
  tokio::spawn(async move {
    loop {
      tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
      let deadline = coarsetime::Clock::now_since_epoch().as_secs() + 4000;
      let mut btree = EXPIRE.lock();
      let keep = btree.split_off(&deadline);
      for host in btree.values() {
        CACHE.remove(host);
      }
      *btree = keep;
    }
  });
});

#[static_init::dynamic]
pub static NOT_EXIST: ExpireSet<String> = ExpireSet::new(30);

pub struct Cert(Ref<'static, String, SslConfig>);

pub async fn get(host: impl Into<String>) -> Result<Option<Cert>> {
  let host = host.into();
  if let Some(ssl_config) = CACHE.get(&host) {
    return Ok(Some(Cert(ssl_config)));
  } else if NOT_EXIST.contains(&host) {
    return Ok(None);
  }
  if let Some((ssl_config, deadline)) = get_by_kvrocks(&host).await? {
    tokio::spawn({
      let host = host.clone();
      async move {
        let mut btree = EXPIRE.lock();
        btree.insert(deadline, host.clone());
      }
    });
    CACHE.insert(host.clone(), ssl_config);
    if let Some(cert) = CACHE.get(&host) {
      return Ok(Some(Cert(cert)));
    }
  }
  NOT_EXIST.insert(host);
  Ok(None)
}

impl std::borrow::Borrow<SslConfig> for Cert {
  fn borrow(&self) -> &SslConfig {
    &self.0
  }
}

impl std::ops::Deref for Cert {
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
    if let Some(cert) = get(host).await? {
      return Ok(Some(cert));
    }
    Ok(None)
  }
}
