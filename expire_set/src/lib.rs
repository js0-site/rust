use std::{
  sync::atomic::{AtomicUsize, Ordering},
  time::Duration,
};

use dashmap::DashSet;
use tokio::task::JoinHandle;

// 用于包装原始指针，手动实现 Send + Sync
struct SendPtr<T>(*const T);

unsafe impl<T> Send for SendPtr<T> {}
unsafe impl<T> Sync for SendPtr<T> {}

impl<T> SendPtr<T> {
  fn new(ptr: *const T) -> Self {
    SendPtr(ptr)
  }

  fn get(&self) -> *const T {
    self.0
  }
}

pub struct ExpireSet<K> {
  pub n: *const AtomicUsize,
  pub cache: *const [DashSet<K>; 2],
  pub timer: JoinHandle<()>,
}

// 手动实现 Send + Sync，因为我们使用的是原始指针
// 这是安全的，因为：
// 1. 指针指向的数据具有 'static 生命周期
// 2. 我们使用 AtomicUsize 来同步访问
// 3. DashSet 本身是线程安全的
unsafe impl<K: Send> Send for ExpireSet<K> {}
unsafe impl<K: Sync> Sync for ExpireSet<K> {}

impl<K: std::hash::Hash + Eq + Clone + Send + Sync + 'static> ExpireSet<K> {
  pub fn new(expire: u64) -> Self {
    // 使用 Box::leak 将数据泄露到 'static 生命周期，获得原始指针
    let cache: *const [DashSet<K>; 2] = Box::leak(Box::new([DashSet::new(), DashSet::new()]));
    let n: *const AtomicUsize = Box::leak(Box::new(AtomicUsize::new(0)));

    // 包装指针使其可以跨线程传递
    let cache_ptr = SendPtr::new(cache);
    let n_ptr = SendPtr::new(n);

    let timer = tokio::spawn(async move {
      loop {
        tokio::time::sleep(Duration::from_secs(expire)).await;
        unsafe {
          let n = n_ptr.get();
          let current = (*n).load(Ordering::Acquire);
          if current > 1 {
            // 手动回收之前泄露的内存
            let _ = Box::from_raw(n as *mut AtomicUsize);
            return;
          }
          let cache = cache_ptr.get();
          let next = (current + 1) % 2;
          (*cache)[next].clear();
          (*n).store(next, Ordering::Relaxed);
        }
      }
    });
    Self { n, cache, timer }
  }

  pub fn insert(&self, key: K) {
    unsafe {
      let idx = (*self.n).load(Ordering::Relaxed);
      (*self.cache)[idx].insert(key);
    }
  }

  pub fn contains(&self, key: &K) -> bool {
    unsafe {
      for set in (*self.cache).iter() {
        if set.contains(key) {
          return true;
        }
      }
    }
    false
  }
}

impl<K> Drop for ExpireSet<K> {
  fn drop(&mut self) {
    unsafe {
      (*self.n).store(usize::MAX, Ordering::Release);
      let _ = Box::from_raw(self.cache as *mut [DashSet<K>; 2]);
    }
  }
}
