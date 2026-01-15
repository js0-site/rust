#!/usr/bin/env node
// Regression testing - keep 512 historical test records
// 回归测试 - 保留 512 个历史测试记录

import { readFileSync, writeFileSync, existsSync } from "fs";
import { getTargetDir } from "./benches/lib.js";

const MAX_HISTORY = 512;
const HISTORY_FILE = "bench_history.json";

const load_history = () => {
  if (!existsSync(HISTORY_FILE)) {
    return [];
  }
  return JSON.parse(readFileSync(HISTORY_FILE, "utf8"));
};

const save_history = (history) => {
  writeFileSync(HISTORY_FILE, JSON.stringify(history, null, 2));
};

const detect_regression = (current, previous) => {
  const regressions = [];
  
  for (const [lib, filters] of Object.entries(current)) {
    if (!previous[lib]) continue;
    
    for (const [filter, metrics] of Object.entries(filters)) {
      if (!previous[lib][filter]) continue;
      
      const prev_metrics = previous[lib][filter];
      
      if (metrics.build?.mean_ns > prev_metrics.build?.mean_ns * 1.1) {
        regressions.push({
          lib,
          filter,
          metric: "build",
          current: metrics.build.mean_ns,
          previous: prev_metrics.build.mean_ns,
          ratio: metrics.build.mean_ns / prev_metrics.build.mean_ns,
        });
      }
      
      if (metrics.contains?.mean_ns > prev_metrics.contains?.mean_ns * 1.1) {
        regressions.push({
          lib,
          filter,
          metric: "contains",
          current: metrics.contains.mean_ns,
          previous: prev_metrics.contains.mean_ns,
          ratio: metrics.contains.mean_ns / prev_metrics.contains.mean_ns,
        });
      }
    }
  }
  
  return regressions;
};

const main = async () => {
  const target_dir = await getTargetDir();
  const json_path = `${target_dir}/criterion/bench_results.json`;
  const current_results = JSON.parse(readFileSync(json_path, "utf8"));
  
  const history = load_history();
  
  if (history.length > 0) {
    const previous = history[history.length - 1];
    const regressions = detect_regression(current_results, previous.results);
    
    if (regressions.length > 0) {
      console.log("⚠️  Performance regressions detected:");
      for (const reg of regressions) {
        console.log(
          `  ${reg.lib}_${reg.filter}.${reg.metric}: ${(reg.ratio * 100 - 100).toFixed(1)}% slower`
        );
      }
    } else {
      console.log("✅ No performance regressions detected");
    }
  }
  
  history.push({
    timestamp: new Date().toISOString(),
    results: current_results,
  });
  
  if (history.length > MAX_HISTORY) {
    history.splice(0, history.length - MAX_HISTORY);
  }
  
  save_history(history);
  console.log(`Saved to history (${history.length}/${MAX_HISTORY} records)`);
};

main();
