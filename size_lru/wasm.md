# SizeLru WebAssembly (Wasm) 集成方案

本方案设计了如何为 `size_lru` 库添加独立的 `wasm` 特性 (Feature) 和独立的 `wasm` 模块，为 JavaScript 环境提供纯 Wasm 版本的缓存绑定。
缓存的设置接口由 JS 侧在调用时显式传入对象大小参数，从而在 Wasm 侧实现对任意 JS 实体（如已加载的 `WebAssembly. Module`、`ImageBitmap` 等）的**零拷贝、零序列化**极速缓存。

---

## 1. Cargo.toml 配置修改

通过引入可选依赖 `wasm-bindgen`，并在 `[features]` 中声明独立的 `wasm` 特性。

```toml
[features]
# 声明独立的 wasm 特性
wasm = ["dep:wasm-bindgen", "lhd"]

[dependencies.wasm-bindgen]
version = "0.2"
optional = true
```

---

## 2. 独立模块设计：`src/wasm.rs`

当开启 `wasm` 特性时，编译器将引入 `wasm` 模块。该模块提供 `WasmCache` 结构体，其 Value 类型为原生的 `wasm_bindgen::JsValue`。

在项目根目录下新建 [src/wasm.rs](file:///Users/z/js0/rust/size_lru/src/wasm.rs) 并写入以下代码：

```rust
//! Wasm bindings for SizeLru cache
//! Wasm 绑定实现

use wasm_bindgen::prelude::*;
use crate::{Lhd, SizeLru};

/// SizeLru cache wrapper for JavaScript environment
/// 适用于 JavaScript 环境的 SizeLru 缓存包装器
#[wasm_bindgen]
pub struct WasmCache {
  // Key 为 String，Value 为原生的 JS 引用（JsValue）
  inner: Lhd<String, JsValue>,
}

#[wasm_bindgen]
impl WasmCache {
  /// Create a new WasmCache with max size
  /// 创建指定最大容量的 WasmCache
  #[wasm_bindgen(constructor)]
  pub fn new(max: usize) -> WasmCache {
    WasmCache {
      inner: Lhd::new(max),
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
```

---

## 3. 入口文件引入：`src/lib.rs`

在 [src/lib.rs](file:///Users/z/js0/rust/size_lru/src/lib.rs) 的模块引入区域添加条件编译指令，只有在启用 `wasm` 特性时才编译该模块：

```rust
#[cfg(feature = "wasm")]
pub mod wasm;
```

---

## 4. 编译与打包命令

使用 `wasm-pack` 将 Rust 代码编译为适用于 Web 浏览器或 Node.js 环境的 Wasm 模块：

```bash
# 编译为适用于 Web bundler (如 Webpack/Vite) 的打包格式，并启用 wasm 特性
wasm-pack build -- --features wasm

# 或者编译为适用于 Node.js 的 CommonJS 格式
wasm-pack build --target nodejs -- --features wasm
```

---

## 5. JavaScript / TypeScript 侧使用示例

```javascript
import { WasmCache } from './pkg/size_lru.js';

// 初始化缓存，比如最大内存权重为 50MB
const cache = new WasmCache(50 * 1024 * 1024);

// 缓存一个动态编译好的 Wasm 模块
async function cacheMyWasmModule(url) {
  const response = await fetch(url);

  // 1. 编译 Wasm 模块（重型黑盒对象）
  const wasmModule = await WebAssembly.compileStreaming(response);

  // 2. JS 侧显式计算大小（例如基于 Wasm 源二进制文件的 Content-Length 并乘上 4 倍 JIT 机器码膨胀系数）
  const originSize = parseInt(response.headers.get('content-length') || '102400');
  const estimatedSize = originSize * 4;

  // 3. 传入 Wasm 缓存，显式提供对象大小
  cache.set(url, wasmModule, estimatedSize);
}

// 获取缓存模块并实例化
function runCachedModule(url) {
  const cachedModule = cache.get(url);
  if (cachedModule) {
    // 零反序列化开销，直接实例化
    return new WebAssembly. Instance(cachedModule, {});
  }
  return null;
}
```