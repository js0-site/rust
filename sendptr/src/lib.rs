#![cfg_attr(docsrs, feature(doc_cfg))]

use std::ops::Deref;

#[derive(Clone)]
pub struct SendPtr<T>(*const T);

unsafe impl<T> Send for SendPtr<T> {}
unsafe impl<T> Sync for SendPtr<T> {}

impl<T> SendPtr<T> {
  pub fn new(ptr: *const T) -> Self {
    SendPtr(ptr)
  }

  pub fn get(&self) -> *const T {
    self.0
  }
}

impl<T> Deref for SendPtr<T> {
  type Target = T;
  fn deref(&self) -> &Self::Target {
    unsafe { &*self.0 }
  }
}
