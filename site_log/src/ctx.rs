use std::{fmt::Debug, net::IpAddr, ops::Deref};

use fred::{clients::Pipeline, prelude::Client};
use ipbin::ipbin;

pub struct Ctx {
  pub ip: IpAddr,
  pub pipe: Pipeline<Client>,
  pub user_id: u64,
  pub site_id: u64,
  pub device_id: u64,
  pub bin: Vec<u8>,
  pub ipbin: Vec<u8>,
  pub site_key: Vec<u8>,
}

impl Deref for Ctx {
  type Target = Pipeline<Client>;
  fn deref(&self) -> &Self::Target {
    &self.pipe
  }
}

impl Debug for Ctx {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Ctx")
      .field("ip", &self.ip)
      .field("user_id", &self.user_id)
      .field("site_id", &self.site_id)
      .field("device_id", &self.device_id)
      .finish()
  }
}

impl Ctx {
  pub fn new(ip: IpAddr, user_id: u64, site_id: u64, device_id: u64) -> Self {
    let site_key = xbin::concat!(b"site:", intbin::to_bin(site_id));
    Self {
      ip,
      user_id,
      site_id,
      device_id,
      site_key,
      ipbin: ipbin(ip),
      bin: vb::e_li([user_id, device_id]),
      pipe: xkv::R.pipeline(),
    }
  }
}
