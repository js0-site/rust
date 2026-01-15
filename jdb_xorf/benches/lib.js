// Common utilities for benchmark processing
// 评测处理公共工具

import { $ } from "zx";

export const getTargetDir = async () => {
  const metadata = await $`cargo metadata --format-version 1`.quiet();
  const json = JSON.parse(metadata.stdout);
  return json.target_directory;
};

export const parseBenchJson = (json_path) => {
  const fs = require("fs");
  const data = JSON.parse(fs.readFileSync(json_path, "utf8"));
  
  const results = {};
  
  for (const bench of data) {
    const { group, function_id, mean, throughput } = bench;
    const [lib, filter_type] = function_id.split("_", 2);
    
    if (!results[lib]) {
      results[lib] = {};
    }
    
    if (!results[lib][filter_type]) {
      results[lib][filter_type] = {};
    }
    
    results[lib][filter_type][group] = {
      mean_ns: mean.point_estimate,
      throughput_ops: throughput ? throughput.per_iteration : null,
    };
  }
  
  return results;
};

export const formatBytes = (bytes) => {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(2)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
};

export const formatTime = (ns) => {
  if (ns < 1000) return `${ns.toFixed(2)} ns`;
  if (ns < 1000000) return `${(ns / 1000).toFixed(2)} μs`;
  if (ns < 1000000000) return `${(ns / 1000000).toFixed(2)} ms`;
  return `${(ns / 1000000000).toFixed(2)} s`;
};

export default { getTargetDir, parseBenchJson, formatBytes, formatTime };
