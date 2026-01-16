#!/usr/bin/env node
import { writeFileSync, mkdirSync } from "fs";
import { getTargetDir, parseCriterion, formatTime, formatBytes } from "./lib.js";
import Table from "cli-table3";

const LANGS = [
  {
    code: "en",
    headers: {
      perf: ["Library", "Filter", "Build Time", "Query Time", "Memory"],
      accuracy: ["Library", "Filter", "False Positive Rate", "False Negative Rate"]
    }
  },
  {
    code: "zh",
    headers: {
      perf: ["库", "过滤器", "构建时间", "查询时间", "内存占用"],
      accuracy: ["库", "过滤器", "假阳率", "假阴率"]
    }
  },
];

const genPerfTable = (results, headers) => {
  let table = `| ${headers.join(" | ")} |\n`;
  table += `| ${headers.map(() => "---").join(" | ")} |\n`;

  // Sort by filter type (8, 16, 32) then by lib name
  // 按过滤器类型（8, 16, 32）然后按库名排序
  const entries = [];
  for (const [lib, filters] of Object.entries(results)) {
    for (const [filter, sizes] of Object.entries(filters)) {
      for (const [size, metrics] of Object.entries(sizes)) {
        entries.push({ lib, filter, size, metrics });
      }
    }
  }
  
  const filterOrder = { 'BinaryFuse8': 0, 'BinaryFuse16': 1, 'BinaryFuse32': 2 };
  entries.sort((a, b) => {
    const orderA = filterOrder[a.filter] ?? 999;
    const orderB = filterOrder[b.filter] ?? 999;
    if (orderA !== orderB) return orderA - orderB;
    return a.lib.localeCompare(b.lib);
  });

  for (const { lib, filter, metrics } of entries) {
    const build_time = formatTime(metrics.build?.mean_ns || 0);
    const query_time = formatTime(metrics.contains?.mean_ns || 0);
    const memory = formatBytes(metrics.memory?.bytes || 0);
    table += `| ${lib} | ${filter} | ${build_time} | ${query_time} | ${memory} |\n`;
  }

  return table;
};

const displayPerfTable = (results, headers) => {
  const table = new Table({
    head: headers,
    chars: {
      'top': '', 'top-mid': '', 'top-left': '', 'top-right': '',
      'bottom': '', 'bottom-mid': '', 'bottom-left': '', 'bottom-right': '',
      'left': '', 'left-mid': '', 'mid': '', 'mid-mid': '',
      'right': '', 'right-mid': '', 'middle': ' '
    },
    style: {
      'head': ['cyan', 'bold'],
      'padding-left': 1,
      'padding-right': 1
    }
  });

  // Sort by filter type (8, 16, 32) then by lib name
  // 按过滤器类型（8, 16, 32）然后按库名排序
  const entries = [];
  for (const [lib, filters] of Object.entries(results)) {
    for (const [filter, sizes] of Object.entries(filters)) {
      for (const [size, metrics] of Object.entries(sizes)) {
        entries.push({ lib, filter, size, metrics });
      }
    }
  }
  
  const filterOrder = { 'BinaryFuse8': 0, 'BinaryFuse16': 1, 'BinaryFuse32': 2 };
  entries.sort((a, b) => {
    const orderA = filterOrder[a.filter] ?? 999;
    const orderB = filterOrder[b.filter] ?? 999;
    if (orderA !== orderB) return orderA - orderB;
    return a.lib.localeCompare(b.lib);
  });

  for (const { lib, filter, metrics } of entries) {
    const build_time = formatTime(metrics.build?.mean_ns || 0);
    const query_time = formatTime(metrics.contains?.mean_ns || 0);
    const memory = formatBytes(metrics.memory?.bytes || 0);
    table.push([lib, filter, build_time, query_time, memory]);
  }

  console.log(table.toString());
};

const genAccuracyTable = (results, headers) => {
  let table = `| ${headers.join(" | ")} |\n`;
  table += `| ${headers.map(() => "---").join(" | ")} |\n`;

  // Sort by filter type (8, 16, 32) then by lib name
  // 按过滤器类型（8, 16, 32）然后按库名排序
  const entries = [];
  for (const [lib, filters] of Object.entries(results)) {
    for (const [filter, sizes] of Object.entries(filters)) {
      for (const [size, metrics] of Object.entries(sizes)) {
        entries.push({ lib, filter, size, metrics });
      }
    }
  }
  
  const filterOrder = { 'BinaryFuse8': 0, 'BinaryFuse16': 1, 'BinaryFuse32': 2 };
  entries.sort((a, b) => {
    const orderA = filterOrder[a.filter] ?? 999;
    const orderB = filterOrder[b.filter] ?? 999;
    if (orderA !== orderB) return orderA - orderB;
    return a.lib.localeCompare(b.lib);
  });

  for (const { lib, filter, metrics } of entries) {
    const fp_rate = (metrics.false_positive?.rate || 0).toFixed(5) + '%';
    const fn_rate = (metrics.false_negative?.rate || 0);
    table += `| ${lib} | ${filter} | ${fp_rate} | ${fn_rate} |\n`;
  }

  return table;
};

const displayAccuracyTable = (results, headers) => {
  const table = new Table({
    head: headers,
    chars: {
      'top': '', 'top-mid': '', 'top-left': '', 'top-right': '',
      'bottom': '', 'bottom-mid': '', 'bottom-left': '', 'bottom-right': '',
      'left': '', 'left-mid': '', 'mid': '', 'mid-mid': '',
      'right': '', 'right-mid': '', 'middle': ' '
    },
    style: {
      'head': ['cyan', 'bold'],
      'padding-left': 1,
      'padding-right': 1
    }
  });

  // Sort by filter type (8, 16, 32) then by lib name
  // 按过滤器类型（8, 16, 32）然后按库名排序
  const entries = [];
  for (const [lib, filters] of Object.entries(results)) {
    for (const [filter, sizes] of Object.entries(filters)) {
      for (const [size, metrics] of Object.entries(sizes)) {
        entries.push({ lib, filter, size, metrics });
      }
    }
  }
  
  const filterOrder = { 'BinaryFuse8': 0, 'BinaryFuse16': 1, 'BinaryFuse32': 2 };
  entries.sort((a, b) => {
    const orderA = filterOrder[a.filter] ?? 999;
    const orderB = filterOrder[b.filter] ?? 999;
    if (orderA !== orderB) return orderA - orderB;
    return a.lib.localeCompare(b.lib);
  });

  for (const { lib, filter, metrics } of entries) {
    const fp_rate = (metrics.false_positive?.rate || 0).toFixed(5) + '%';
    const fn_rate = (metrics.false_negative?.rate || 0);
    table.push([lib, filter, fp_rate, fn_rate]);
  }

  console.log(table.toString());
};

const main = async () => {
  const target_dir = await getTargetDir();
  const results = parseCriterion(target_dir);

  mkdirSync("readme", { recursive: true });

  await Promise.all(
    LANGS.map(({ code, headers }) => {
      const perfTable = genPerfTable(results, headers.perf);
      const accuracyTable = genAccuracyTable(results, headers.accuracy);
      
      const content = `## Performance Benchmark\n\n${perfTable}\n## Accuracy\n\n${accuracyTable}`;
      const path = `readme/${code}.bench.md`;
      writeFileSync(path, content);
      console.log(`  ${path}`);
    })
  );

  console.log("\nGenerated benchmark tables\n");
  
  // Display tables to console using cli-table3
  LANGS.forEach(({ code, headers }) => {
    console.log(`\n${code.toUpperCase()} - Performance Benchmark:`);
    displayPerfTable(results, headers.perf);
    console.log(`\n${code.toUpperCase()} - Accuracy:`);
    displayAccuracyTable(results, headers.accuracy);
  });
};

main();
