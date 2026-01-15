import { readFileSync, readdirSync, existsSync, statSync } from "fs";
import { join } from "path";
import { $ } from "zx";

export const getTargetDir = async () => {
  const metadata = await $`cargo metadata --format-version 1`.quiet();
  const json = JSON.parse(metadata.stdout);
  return json.target_directory;
};

export const parseCriterion = (target_dir) => {
  const results = {};
  const criterion_dir = `${target_dir}/criterion`;
  
  if (!existsSync(criterion_dir)) {
    console.log(`Warning: Criterion directory not found: ${criterion_dir}`);
    return results;
  }
  
  const groups = ["build", "contains", "false_positive"];
  
  for (const group of groups) {
    const group_dir = join(criterion_dir, group);
    if (!existsSync(group_dir)) continue;
    
    const benchmarks = readdirSync(group_dir);
    
    for (const bench of benchmarks) {
      const bench_dir = join(group_dir, bench);
      if (!statSync(bench_dir).isDirectory()) continue;
      
      const sizes = readdirSync(bench_dir);
      
      for (const size of sizes) {
        const estimates_file = join(bench_dir, size, "base", "estimates.json");
        if (!existsSync(estimates_file)) continue;
        
        const estimates = JSON.parse(readFileSync(estimates_file, "utf8"));
        const mean_ns = estimates.mean.point_estimate;
        
        const parts = bench.split("_");
        const lib = parts[0];
        const filter = parts.slice(1).join("_");
        
        if (!results[lib]) results[lib] = {};
        if (!results[lib][filter]) results[lib][filter] = {};
        if (!results[lib][filter][size]) results[lib][filter][size] = {};
        
        results[lib][filter][size][group] = { mean_ns };
      }
    }
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
