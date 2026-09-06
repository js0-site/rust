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
