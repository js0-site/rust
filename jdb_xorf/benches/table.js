#!/usr/bin/env node
import { writeFileSync, mkdirSync } from "fs";
import { getTargetDir, parseCriterion, formatTime } from "./lib.js";

const genTable = (results, lang) => {
  const is_zh = lang === "zh";
  const headers = is_zh
    ? ["库", "过滤器", "大小", "构建时间", "查询时间", "假阳率"]
    : ["Library", "Filter", "Size", "Build Time", "Query Time", "FP Rate"];

  let table = `| ${headers.join(" | ")} |\n`;
  table += `| ${headers.map(() => "---").join(" | ")} |\n`;

  for (const [lib, filters] of Object.entries(results)) {
    for (const [filter, sizes] of Object.entries(filters)) {
      for (const [size, metrics] of Object.entries(sizes)) {
        const build_time = formatTime(metrics.build?.mean_ns || 0);
        const query_time = formatTime(metrics.contains?.mean_ns || 0);
        const fp_rate = metrics.false_positive?.rate || "N/A";

        table += `| ${lib} | ${filter} | ${size} | ${build_time} | ${query_time} | ${fp_rate} |\n`;
      }
    }
  }

  return table;
};

const main = async () => {
  const target_dir = await getTargetDir();
  const results = parseCriterion(target_dir);

  mkdirSync("readme", { recursive: true });

  await Promise.all(
    ["en", "zh"].map((lang) => {
      const table = genTable(results, lang);
      const path = `readme/${lang}.bench.md`;
      writeFileSync(path, table);
      console.log(`  ${path}`);
    })
  );

  console.log("Generated benchmark tables");
};

main();
