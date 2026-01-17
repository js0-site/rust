//! Global ID generator
//! 全局 ID 生成器

use std::cell::UnsafeCell;

use crate::Ider;

thread_local! {
  static IDER: UnsafeCell<Ider> = UnsafeCell::new(Ider::new());
}

/// Generate unique ID
/// 生成唯一 ID
#[inline]
pub fn id() -> u64 {
  IDER.with(|ider| unsafe { (*ider.get()).get() })
}

/// Initialize ID generator with base ID
/// 用基础 ID 初始化 ID 生成器
#[inline]
pub fn id_init(base: u64) {
  IDER.with(|ider| unsafe { (*ider.get()).init(base) });
}
