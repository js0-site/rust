#!/usr/bin/env node
import { readFileSync, writeFileSync, existsSync, mkdirSync } from "fs";
import { getTargetDir, parseCriterion } from "./benches/lib.js";

const MAX_HISTORY = 512;
const HISTORY_FILE = "bench_history.json";

const loadHistory = () => {
  if (!existsSync(HISTORY_FILE)) {
    return [];
  }
  return JSON.parse(readFileSync(HISTORY_FILE, "utf8"));
};

const saveHistory = (history) => {
  writeFileSync(HISTORY_FILE, JSON.stringify(history, null, 2));
};

const detectRegression = (current, previous) => {
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
  const current_results = parseCriterion(target_dir);
  
  const history = loadHistory();
  
  if (history.length > 0) {
    const previous = history[history.length - 1];
    const regressions = detectRegression(current_results, previous.results);
    
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
  
  saveHistory(history);
  console.log(`Saved to history (${history.length}/${MAX_HISTORY} records)`);
};

main();
