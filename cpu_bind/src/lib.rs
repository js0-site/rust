#![cfg_attr(docsrs, feature(doc_cfg))]

use std::{
  collections::HashSet,
  io,
  thread::{self, JoinHandle},
};

use compio_runtime::{Runtime, RuntimeBuilder};
use core_affinity::CoreId;
use cpu_load::CPU_LOAD;

/// Spawn a thread with compio runtime bound to the idlest CPU
/// 在最空闲的 CPU 上创建带 compio 运行时的线程
pub fn spawn<F, T>(f: F) -> JoinHandle<io::Result<T>>
where
  F: FnOnce(&Runtime) -> T + Send + 'static,
  T: Send + 'static,
{
  spawn_on(CPU_LOAD.idlest(), f)
}

/// Spawn a thread with compio runtime bound to a specific CPU
/// 在指定 CPU 上创建带 compio 运行时的线程
fn spawn_on<F, T>(core_id: usize, f: F) -> JoinHandle<io::Result<T>>
where
  F: FnOnce(&Runtime) -> T + Send + 'static,
  T: Send + 'static,
{
  thread::spawn(move || {
    let rt = RuntimeBuilder::new()
      .thread_affinity(HashSet::from([core_id]))
      .build()?;
    Ok(f(&rt))
  })
}

/// Bind current thread to the idlest CPU, returns the core id
/// 将当前线程绑定到最空闲的 CPU，返回核心 id
pub fn bind() -> usize {
  let core_id = CPU_LOAD.idlest();
  core_affinity::set_for_current(CoreId { id: core_id });
  core_id
}
