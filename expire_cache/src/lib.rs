#![cfg_attr(docsrs, feature(doc_cfg))]

use std::{
  borrow::Borrow,
  hash::Hash,
  sync::atomic::{AtomicBool, AtomicUsize, Ordering},
  time::Duration,
};

use boxleak::boxleak;
use parking_lot::{Mutex, Once};
use sendptr::SendPtr;
use tokio::{task::AbortHandle, time::sleep};

#[cfg(feature = "hashmap")]
pub mod map;
#[cfg(feature = "hashset")]
pub mod set;

/// Trait for cache storage / 缓存存储 trait
pub trait Map: Default + Send + Sync {
  type Key;
  type Val;
  type RefVal<'a>
  where
    Self: 'a;

  fn clear(&self);
  fn insert(&self, key: Self::Key, val: Self::Val);
  fn get<'a, Q>(&'a self, key: &Q) -> Option<Self::RefVal<'a>>
  where
    Self::Key: Borrow<Q>,
    Q: Hash + Eq + ?Sized;
}

#[cfg(feature = "get_or_init_async")]
mod get_or_init_async;

#[cfg(feature = "get_or_init")]
mod get_or_init;
#[cfg(feature = "get_or_init")]
pub use get_or_init::GetOrInit;

struct Inner<T> {
  pos: AtomicUsize,
  // Box for alignment (papaya/seize needs 128-byte) / 用 Box 确保对齐
  cache: [Box<T>; 2],
  abort: Mutex<Option<AbortHandle>>,
}

/// Generational cache with expiration / 带过期的分代缓存
pub struct Expire<T: Map> {
  inner: *const Inner<T>,
  expire: u64,
  started: AtomicBool,
  once: Once,
}

unsafe impl<T: Map> Send for Expire<T> {}
unsafe impl<T: Map> Sync for Expire<T> {}

impl<T: Map + 'static> Expire<T> {
  fn inner(&self) -> &Inner<T> {
    unsafe { &*self.inner }
  }

  fn ensure_timer(&self) {
    if self.started.load(Ordering::Relaxed) {
      return;
    }
    self.once.call_once(|| {
      let ptr = SendPtr::new(self.inner);
      let expire = self.expire;
      let handle = tokio::spawn(async move {
        loop {
          sleep(Duration::from_secs(expire)).await;
          unsafe {
            let inner = &*ptr.get();
            let old = inner.pos.load(Ordering::Acquire);
            let n = 1 - old;
            inner.pos.store(n, Ordering::Release);
            sleep(Duration::from_millis(100)).await;
            inner.cache[old].clear();
          }
        }
      });
      *self.inner().abort.lock() = Some(handle.abort_handle());
      self.started.store(true, Ordering::Relaxed);
    });
  }

  pub fn get<'a, Q>(&'a self, key: &Q) -> Option<T::RefVal<'a>>
  where
    T::Key: Borrow<Q>,
    Q: Hash + Eq + ?Sized,
  {
    self.ensure_timer();
    let inner = self.inner();
    let pos = inner.pos.load(Ordering::Relaxed);
    inner.cache[pos]
      .get(key)
      .or_else(|| inner.cache[1 - pos].get(key))
  }

  pub fn insert(&self, key: T::Key, val: T::Val) {
    self.ensure_timer();
    let inner = self.inner();
    let idx = inner.pos.load(Ordering::Acquire);
    inner.cache[idx].insert(key, val)
  }

  pub fn new(expire: u64) -> Self {
    let inner = boxleak(Inner {
      pos: AtomicUsize::new(0),
      cache: [Box::new(T::default()), Box::new(T::default())],
      abort: Mutex::new(None),
    });

    Self {
      inner,
      expire,
      started: AtomicBool::new(false),
      once: Once::new(),
    }
  }
}

impl<T: Map> Drop for Expire<T> {
  fn drop(&mut self) {
    let inner = unsafe { &*self.inner };
    if let Some(handle) = inner.abort.lock().take() {
      handle.abort();
    }
    unsafe {
      let _ = Box::from_raw(self.inner as *mut Inner<T>);
    }
  }
}
