[English](#en) | [中文](#zh)

---

<a id="en"></a>

# @3-/lru : Fastest Size-Aware LRU Cache (Wasm)

Install the package via Bun:

```bash
bun i @3-/lru
```

## Usage Example

```javascript
import init, { WasmCache } from "@3-/lru";

// Initialize the WebAssembly module
await init();

// Initialize cache and binary items
const cache = new WasmCache(250),
  bin_a = new Uint8Array([1, 2, 3, 4]),
  bin_b = new Uint8Array([5, 6, 7, 8]);
cache.set("a", bin_a, bin_a.length); // size = 4 + 96 = 100
cache.set("b", bin_b, bin_b.length); // size = 4 + 96 = 100

// Retrieve cached items
console.log("Get a:", cache.get("a"));
console.log("Get b:", cache.get("b"));

// Insert an item that triggers eviction of 'a' (100 + 100 + 100 = 300 > 250)
const bin_c = new Uint8Array([9, 10, 11, 12]);
cache.set("c", bin_c, bin_c.length); // size = 4 + 96 = 100

// Verify eviction of 'a' and retention of 'b' and 'c'
console.log("Get a after eviction:", cache.get("a")); // undefined
console.log("Get b after eviction:", cache.get("b")); // Uint8Array
```

## Eviction Callback

You can provide an optional eviction callback function when instantiating `WasmCache`. The callback will be triggered synchronously with the evicted key and value:

```javascript
const cacheWithCallback = new WasmCache(250, (key, value) => {
  console.log(`Evicted: key="${key}", value=`, value);
});

cacheWithCallback.set("a", bin_a, bin_a.length);
cacheWithCallback.set("b", bin_b, bin_b.length);

// Trigger eviction
cacheWithCallback.set("c", bin_c, bin_c.length);
```

---

Underlying Rust Crate: [size_lru](https://crates.io/crates/size_lru)

---

<a id="zh"></a>

# @3-/lru : 最快的大小感知 LRU 缓存 (Wasm)

通过 Bun 安装：

```bash
bun i @3-/lru
```

## 使用演示

```javascript
import init, { WasmCache } from "@3-/lru";

// 初始化 WebAssembly 模块
await init();

// 初始化缓存并写入二进制数据
const cache = new WasmCache(250),
  bin_a = new Uint8Array([1, 2, 3, 4]),
  bin_b = new Uint8Array([5, 6, 7, 8]);
cache.set("a", bin_a, bin_a.length); // 实际大小 = 4 + 96 = 100
cache.set("b", bin_b, bin_b.length); // 实际大小 = 4 + 96 = 100

// 获取缓存数据
console.log("获取 a:", cache.get("a"));
console.log("获取 b:", cache.get("b"));

// 插入新数据触发淘汰（300 > 250，因为之后访问了 'b'，'a' 最久未被访问将被淘汰）
const bin_c = new Uint8Array([9, 10, 11, 12]);
cache.set("c", bin_c, bin_c.length); // 实际大小 = 4 + 96 = 100

// 验证淘汰结果
console.log("淘汰后获取 a:", cache.get("a")); // undefined
console.log("淘汰后获取 b:", cache.get("b")); // Uint8Array
```

## 淘汰回调

你可以在实例化 `WasmCache` 时传入一个可选的回调函数。当有条目被淘汰时，该函数会被同步触发，并传入被淘汰的键和值：

```javascript
const cacheWithCallback = new WasmCache(250, (key, value) => {
  console.log(`条目被淘汰: 键="${key}", 值=`, value);
});

cacheWithCallback.set("a", bin_a, bin_a.length);
cacheWithCallback.set("b", bin_b, bin_b.length);

// 插入新数据触发淘汰
cacheWithCallback.set("c", bin_c, bin_c.length);
```

---

底层 Rust Crate: [size_lru](https://crates.io/crates/size_lru)
