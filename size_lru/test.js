#!/usr/bin/env bun

import init, { WasmCache } from "./pkg/index.js";

// Initialize the WebAssembly module
// 初始化 WebAssembly 模块
await init();

// Initialize cache and binary items
// 初始化缓存与二进制数据
const cache = new WasmCache(250),
  bin_a = new Uint8Array([1, 2, 3, 4]),
  bin_b = new Uint8Array([5, 6, 7, 8]);
cache.set("a", bin_a, bin_a.length); // size = 4 + 96 = 100
cache.set("b", bin_b, bin_b.length); // size = 4 + 96 = 100

// Retrieve and log cached items (this makes 'a' most recently used)
// 获取并打印缓存中的条目（这使 'a' 变为最近最常使用）
console.log("获取 a:", cache.get("a"));
console.log("获取 b:", cache.get("b"));
console.log("当前总大小:", cache.size()); // Should be 200 / 应该为 200

// Insert an item that triggers eviction of 'a' (100 + 100 + 100 = 300 > 250, 'a' is least recently used since 'b' was accessed last)
// 插入超容数据触发淘汰 'a' (100 + 100 + 100 = 300 > 250，因为之后访问了 'b'，'a' 最久未被访问)
const bin_c = new Uint8Array([9, 10, 11, 12]);
cache.set("c", bin_c, bin_c.length); // size = 4 + 96 = 100

// Verify eviction of 'a' and retention of 'b' and 'c'
// 验证 'a' 已被淘汰，而 'b' 与 'c' 仍保留
console.log("淘汰后获取 a:", cache.get("a")); // Should be undefined / 应该为 undefined
console.log("淘汰后获取 b:", cache.get("b")); // Should be bin_b / 应该为 bin_b
console.log("淘汰后获取 c:", cache.get("c")); // Should be bin_c / 应该为 bin_c
console.log("当前总大小:", cache.size()); // Should be 200 / 应该为 200
console.log("当前条目数量:", cache.len()); // Should be 2 / 应该为 2

// Test eviction callback
// 测试淘汰回调
console.log("\n--- 测试淘汰回调 ---");
const evicted = [];
const cacheWithCallback = new WasmCache(250, (key, val) => {
  console.log(`回调触发 - 淘汰键: ${key}, 值:`, val);
  evicted.push({ key, val });
});

cacheWithCallback.set("a", bin_a, bin_a.length);
cacheWithCallback.set("b", bin_b, bin_b.length);
// Make 'a' MRU, then 'b' MRU
cacheWithCallback.get("a");
cacheWithCallback.get("b");

// This should evict 'a'
cacheWithCallback.set("c", bin_c, bin_c.length);

console.log("已淘汰的条目:", evicted);
if (evicted.length !== 1) {
  throw new Error("淘汰回调未触发！");
}
const { key, val } = evicted[0];
if (key === "a") {
  if (val !== bin_a) throw new Error("淘汰的值不匹配！");
} else if (key === "b") {
  if (val !== bin_b) throw new Error("淘汰的值不匹配！");
} else {
  throw new Error(`未知的淘汰键: ${key}`);
}
console.log("淘汰回调测试成功！");
