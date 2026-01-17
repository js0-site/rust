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
  // Use workspace target for criterion data
  // 使用工作区 target 获取 criterion 数据
  const criterion_dir = `${target_dir}/criterion`;

  if (!existsSync(criterion_dir)) {
    console.log(`Warning: Criterion directory not found: ${criterion_dir}`);
    return results;
  }

  const groups = ["build", "has"];

  for (const group of groups) {
    const group_dir = join(criterion_dir, group);
    if (!existsSync(group_dir)) continue;

    const benchmarks = readdirSync(group_dir);

    for (const bench of benchmarks) {
      if (!bench.includes("jdb") && !bench.includes("xorf")) continue;
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

  // Read false positive, false negative rates and memory from local target
  // 从本地 target 读取假阳率、假阴率和内存数据
  const rates_dir = "target/criterion/rates";
  if (existsSync(rates_dir)) {
    const rate_files = readdirSync(rates_dir);

    for (const file of rate_files) {
      const match = file.match(/^(fp|fn|mem)_(.+)_(\d+)\.txt$/);
      if (!match) continue;

      const [, type, filter_name, size] = match;
      const content = readFileSync(join(rates_dir, file), "utf8").trim();

      const parts = filter_name.split("_");
      const lib = parts[0];
      const filter = parts.slice(1).join("_");

      if (!results[lib]) results[lib] = {};
      if (!results[lib][filter]) results[lib][filter] = {};
      if (!results[lib][filter][size]) results[lib][filter][size] = {};

      if (type === "fp") {
        results[lib][filter][size].false_positive = {
          rate: parseFloat(content),
        };
      } else if (type === "fn") {
        results[lib][filter][size].false_negative = {
          rate: parseFloat(content),
        };
      } else if (type === "mem") {
        results[lib][filter][size].memory = { bytes: parseInt(content, 10) };
      }
    }
  }

  return results;
};

export const formatBytes = (bytes) => {
  if (bytes === 0) return "0 B";
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
export const normalizeFilter = (name) => name.replace("BinaryFuse", "Bf");

export const filterOrder = {
  Bf8: 0,
  BinaryFuse8: 0,
  Bf16: 1,
  BinaryFuse16: 1,
  Bf32: 2,
  BinaryFuse32: 2,
};

export const getFilterScore = (filterName) => {
  return filterOrder[filterName] ?? 999;
};

export const compareFilters = (a, b) => {
  const orderA = getFilterScore(a);
  const orderB = getFilterScore(b);
  return orderA - orderB;
};
