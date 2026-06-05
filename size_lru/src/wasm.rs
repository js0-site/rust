//! Wasm bindings for SizeLru cache
//! Wasm 绑定实现

use wasm_bindgen::prelude::{wasm_bindgen, JsValue};

use crate::{Lhd, OnRm};

/// Callback handler for WasmCache to bridge to JS functions
/// WasmCache 的回调处理器，桥接 JS 函数
pub struct WasmOnRm {
  cb: Option<js_sys::Function>,
}

impl OnRm<String, Lhd<String, JsValue, WasmOnRm>> for WasmOnRm {
  fn call(&mut self, key: &String, cache: &Lhd<String, JsValue, WasmOnRm>) {
    if let Some(ref cb) = self.cb {
      let val = cache.peek(key).cloned().unwrap_or_else(JsValue::undefined);
      let this = JsValue::null();
      let key_js = JsValue::from_str(key);
      let _ = cb.call2(&this, &key_js, &val);
    }
  }
}

/// SizeLru cache wrapper for JavaScript environment
/// 适用于 JavaScript 环境的 SizeLru 缓存包装器
#[wasm_bindgen]
pub struct WasmCache {
  // Key 为 String，Value 为原生的 JS 引用（JsValue）
  inner: Lhd<String, JsValue, WasmOnRm>,
}

#[wasm_bindgen]
impl WasmCache {
  /// Create a new WasmCache with max size and optional eviction callback
  /// 创建指定最大容量与可选淘汰回调的 WasmCache
  #[wasm_bindgen(constructor)]
  pub fn new(max: usize, on_rm: Option<js_sys::Function>) -> WasmCache {
    WasmCache {
      inner: Lhd::with_on_rm(max, WasmOnRm { cb: on_rm }),
    }
  }

  /// Insert a JS object into the cache with an explicit size weight
  /// 插入任意 JS 对象并指定大小权重
  #[wasm_bindgen]
  pub fn set(&mut self, key: String, val: JsValue, size: u32) {
    self.inner.set(key, val, size);
  }

  /// Retrieve the original JS object reference (Zero-copy, O(1))
  /// 获取缓存对象的原生 JS 引用（零拷贝，O(1)）
  #[wasm_bindgen]
  pub fn get(&mut self, key: &str) -> Option<JsValue> {
    self.inner.get(key).cloned()
  }

  /// Remove a cached object by key
  /// 移除指定键的对象
  #[wasm_bindgen]
  pub fn rm(&mut self, key: &str) {
    self.inner.rm(key);
  }

  /// Peek at a value without updating stats
  /// 查看缓存值但不更新统计
  #[wasm_bindgen]
  pub fn peek(&self, key: &str) -> Option<JsValue> {
    self.inner.peek(key).cloned()
  }

  /// Check if the cache is empty
  /// 检查缓存是否为空
  #[wasm_bindgen(js_name = isEmpty)]
  pub fn is_empty(&self) -> bool {
    self.inner.is_empty()
  }

  /// Get the number of entries in the cache
  /// 获取当前缓存的条目数量
  #[wasm_bindgen]
  pub fn len(&self) -> usize {
    self.inner.len()
  }

  /// Get the current total size (weight) of the cache
  /// 获取当前缓存占用的总大小（权重）
  #[wasm_bindgen]
  pub fn size(&self) -> usize {
    self.inner.size()
  }
}
