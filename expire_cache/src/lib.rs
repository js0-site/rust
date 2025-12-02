#![cfg_attr(docsrs, feature(doc_cfg))]

use std::{sync::atomic::AtomicI8, time::Duration};

use boxleak::boxleak;
use sendptr::SendPtr;
use set_timer::set_timer;
use tokio::task::JoinHandle;
use ts_::sec;

pub trait Map: Default + Send + Sync {
  type Key;
  type Val;
  fn clear(&self);
  fn get(&self, key: &Self::Key) -> Option<Self::Val>;
  fn insert(&self, key: &Self::Key, val: Self::Val);
}

pub struct Expire<T: Map> {
  pub cache_a: *const T,
  pub cache_b: *const T,
  pub cache_now: *const T,
  pub n: *const AtomicI8,
  pub timer: JoinHandle<()>,
}

unsafe impl<T: Map> Send for Expire<T> {}
unsafe impl<T: Map> Sync for Expire<T> {}

impl<T: Map> Expire<T> {
  pub fn get(&self, key: &T::Key) -> Option<T::Val> {
    let v = self.cache_a.get(key);
    if v.is_some() {
      v
    } else {
      self.cache_b.get(key)
    }
  }

  pub fn insert(&self, key: &T::Key, val: T::Val) {
    self.cache_now.insert(key, val)
  }

  pub fn new(expire: u64) -> Self {
    let cache_a = boxleak(Default::default());
    let cache_b = boxleak(Default::default());
    let n = boxleak(AtomicI8::new(0));
    let n_ptr = SendPtr::new(n);
    let a = SendPtr::new(cache_a);
    let b = SendPtr::new(cache_b);
    Self {
      cache_a,
      cache_b,
      cache_now: cache_a,
      n,
      timer: set_timer(
        || {
          let n: *const AtomicUsize = n_ptr.get();
          let current: usize = (*n).load(Ordering::Acquire);
        },
        Duration::from_secs(expire),
      ),
    }
  }
}

impl Drop for Expire<T> {
  fn drop(&mut self) {
    self.timer.abort();
  }
}
