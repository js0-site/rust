#![cfg_attr(docsrs, feature(doc_cfg))]

use std::{
  borrow::Borrow,
  sync::atomic::{AtomicUsize, Ordering},
  time::Duration,
};

use boxleak::boxleak;
use sendptr::SendPtr;
use tokio::task::JoinHandle;

#[cfg(feature = "dashmap")]
pub mod map;
#[cfg(feature = "dashset")]
mod set;

pub trait Map: Default + Send + Sync {
  type Key;
  type Val;
  type RefVal<'a>
  where
    Self: 'a;
  fn clear(&self);
  fn insert(&self, key: Self::Key, val: Self::Val);
  fn get<'a>(&'a self, key: &Self::Key) -> Option<Self::RefVal<'a>>;
}

#[cfg(feature = "get_or_init_async")]
mod get_or_init_async;
#[cfg(feature = "get_or_init_async")]
pub use get_or_init_async::GetOrInitAsync;

#[cfg(feature = "get_or_init")]
mod get_or_init;
#[cfg(feature = "get_or_init")]
pub use get_or_init::GetOrInit;

struct Inner<T> {
  pos: AtomicUsize,
  cache: [T; 2],
}

pub struct Expire<T: Map> {
  inner: *const Inner<T>,
  timer: JoinHandle<()>,
}

unsafe impl<T: Map> Send for Expire<T> {}
unsafe impl<T: Map> Sync for Expire<T> {}

impl<T: Map + 'static> Expire<T> {
  pub fn get<'a>(&'a self, key: impl Borrow<T::Key>) -> Option<T::RefVal<'a>> {
    let key = key.borrow();
    let inner = unsafe { &*self.inner };
    let pos = inner.pos.load(Ordering::Relaxed);
    if let Some(v) = inner.cache[pos].get(key) {
      return Some(v);
    }
    inner.cache[1 - pos].get(key)
  }

  pub fn insert(&self, key: T::Key, val: T::Val) {
    let inner = unsafe { &*self.inner };
    let idx = inner.pos.load(Ordering::Acquire);
    inner.cache[idx].insert(key, val)
  }

  pub fn new(expire: u64) -> Self {
    let inner = boxleak(Inner {
      pos: AtomicUsize::new(0),
      cache: [T::default(), T::default()],
    });
    let inner_ptr = SendPtr::new(inner);

    Self {
      inner,
      timer: tokio::spawn(async move {
        defer_lite::defer! {
          unsafe {
            let _ = Box::from_raw(inner_ptr.get() as *mut Inner<T>);
          }
        }
        loop {
          tokio::time::sleep(Duration::from_secs(expire)).await;
          unsafe {
            let inner = &*inner_ptr.get();
            let n = (inner.pos.load(Ordering::Relaxed) + 1) % 2;
            inner.cache[n].clear();
            inner.pos.store(n, Ordering::Release);
          }
        }
      }),
    }
  }
}

impl<T: Map> Drop for Expire<T> {
  fn drop(&mut self) {
    self.timer.abort();
  }
}
