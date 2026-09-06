#![cfg_attr(docsrs, feature(doc_cfg))]

use std::{cell::UnsafeCell, fmt, ops::Deref};

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

impl fmt::Display for GlobalStr {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(self.deref())
  }
}

impl fmt::Debug for GlobalStr {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(self.deref())
  }
}

impl Deref for GlobalStr {
  type Target = str;
  fn deref(&self) -> &Self::Target {
    unsafe { *self.0.get() }
  }
}
