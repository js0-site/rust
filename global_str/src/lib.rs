#![cfg_attr(docsrs, feature(doc_cfg))]

use std::{cell::UnsafeCell, ops::Deref};

pub struct GlobalStr(pub UnsafeCell<&'static str>);

impl GlobalStr {
  pub const fn new() -> Self {
    Self(UnsafeCell::new(""))
  }
}

impl Default for GlobalStr {
  fn default() -> Self {
    Self::new()
  }
}

unsafe impl Sync for GlobalStr {}
unsafe impl Send for GlobalStr {}

impl std::fmt::Display for GlobalStr {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(self.deref())
  }
}

impl std::fmt::Debug for GlobalStr {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(self.deref())
  }
}

impl Deref for GlobalStr {
  type Target = str;
  fn deref(&self) -> &Self::Target {
    unsafe { *self.0.get() }
  }
}
