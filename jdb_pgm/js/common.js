//! Shared constants and utilities for benchmark scripts
//! 基准测试脚本的共享常量和工具函数

export const ALGORITHM_COLORS = {
  binary_search: "#3b82f6",
  btreemap: "#f59e0b",
  hashmap: "#ef4444",
  jdb_pgm: "#10b981",
  external_pgm: "#f97316",
};

export const ALGORITHM_NAMES = {
  binary_search: "Binary Search",
  btreemap: "BTreeMap",
  hashmap: "HashMap",
  jdb_pgm: "jdb_pgm",
  external_pgm: "pgm_index",
};

export const ALGORITHM_NAMES_ZH = {
  binary_search: "二分查找",
  btreemap: "BTreeMap",
  hashmap: "HashMap",
  jdb_pgm: "jdb_pgm",
  external_pgm: "pgm_index",
};

export const EPSILON_EXPLANATIONS = {
  en: {
    title: "*Epsilon (ε) controls the accuracy-speed trade-off:*",
    definition:
      "*Mathematical definition: ε defines the maximum absolute error between the predicted position and the actual position in the data array. When calling `load(data, epsilon, ...)`, ε guarantees |pred - actual| ≤ ε, where positions are indices within the data array of length `data.len()`.*",
    example:
      "*Example: For 1M elements with ε=32, if the actual key is at position 1000:*",
    examples: [
      "ε=32 predicts position between 968-1032, then checks up to 64 elements",
      "ε=128 predicts position between 872-1128, then checks up to 256 elements",
    ],
  },
  zh: {
    title: "*Epsilon (ε) 控制精度与速度的权衡：*",
    definition:
      "*数学定义：ε 定义了预测位置与实际位置在数据数组中的最大绝对误差。调用 `load(data, epsilon, ...)` 时，ε 保证 |pred - actual| ≤ ε，其中位置是长度为 `data.len()` 的数据数组中的索引。*",
    example: "*举例说明：对于 100 万个元素，ε=32 时，如果实际键在位置 1000：*",
    examples: [
      "ε=32 预测位置在 968-1032 之间，然后检查最多 64 个元素",
      "ε=128 预测位置在 872-1128 之间，然后检查最多 256 个元素",
    ],
  },
};

export const getColor = (algo) => ALGORITHM_COLORS[algo] || "#a1a1aa";

export const fmtTime = (ns) => {
  if (ns >= 1e9) return `${(ns / 1e9).toFixed(2)}s`;
  if (ns >= 1e6) return `${(ns / 1e6).toFixed(2)}ms`;
  if (ns >= 1e3) return `${(ns / 1e3).toFixed(2)}µs`;
  return `${ns.toFixed(2)}ns`;
};

export const fmtThroughput = (ops) => {
  if (ops >= 1e6) return `${(ops / 1e6).toFixed(2)}M/s`;
  if (ops >= 1e3) return `${(ops / 1e3).toFixed(2)}K/s`;
  return `${ops.toFixed(2)}/s`;
};

export const groupByDataSize = (results) => {
  const grouped = {};
  for (const r of results) {
    if (!grouped[r.data_size]) {
      grouped[r.data_size] = [];
    }
    grouped[r.data_size].push(r);
  }
  return grouped;
};

export const getSortedDataSizes = (grouped) => {
  return Object.entries(grouped).sort(
    (a, b) => parseInt(a[0]) - parseInt(b[0]),
  );
};

export const formatDataSize = (size) => parseInt(size).toLocaleString();

export const formatMemory = (bytes) => {
  if (bytes >= 1024 * 1024 * 1024)
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
  if (bytes >= 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
  if (bytes >= 1024) return `${(bytes / 1024).toFixed(2)} KB`;
  return `${bytes} B`;
};

