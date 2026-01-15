#!/usr/bin/env node
// Generate benchmark result tables
// 生成评测结果表格

import { readFileSync, writeFileSync } from "fs";
import { getTargetDir, formatTime, formatBytes } from "./lib.js";

const gen_table = (results, lang) => {
  const is_zh = lang === "zh";
  const headers = is_zh
    ? ["库", "过滤器", "构建时间", "查询时间", "假阳率", "内存"]
    : ["Library", "Filter", "Build Time", "Query Time", "FP Rate", "Memory"];

  let table = `| ${headers.join(" | ")} |\n`;
  table += `| ${headers.map(() => "---").join(" | ")} |\n`;

  for (const [lib, filters] of Object.entries(results)) {
    for (const [filter, metrics] of Object.entries(filters)) {
      const build_time = formatTime(metrics.build?.mean_ns || 0);
      const query_time = formatTime(metrics.contains?.mean_ns || 0);
      const fp_rate = metrics.false_positive?.rate || "N/A";
      const memory = formatBytes(metrics.size_bytes || 0);

      table += `| ${lib} | ${filter} | ${build_time} | ${query_time} | ${fp_rate} | ${memory} |\n`;
    }
  }

  return table;
};

const main = async () => {
  const target_dir = await getTargetDir();
  const json_path = `${target_dir}/criterion/bench_results.json`;
  const results = JSON.parse(readFileSync(json_path, "utf8"));

  const en_table = gen_table(results, "en");
  const zh_table = gen_table(results, "zh");

  writeFileSync("readme/en.bench.md", en_table);
  writeFileSync("readme/zh.bench.md", zh_table);

  console.log("Generated benchmark tables");
};

main();
