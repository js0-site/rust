#!/usr/bin/env node
import { writeFileSync, mkdirSync } from "fs";
import {
  getTargetDir,
  parseCriterion,
  formatTime,
  formatBytes,
} from "./lib.js";
import { LANG, RESOURCES } from "./i18n/index.js";
import Table from "cli-table3";

const LANGS = Object.entries(RESOURCES).map(([code, labels]) => ({
  code,
  titles: {
    perf: labels.table_title_perf,
    accuracy: labels.table_title_accuracy,
  },
  headers: {
    perf: labels.table_headers_perf,
    accuracy: labels.table_headers_accuracy,
  },
}));

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

  const filterOrder = { BinaryFuse8: 0, BinaryFuse16: 1, BinaryFuse32: 2 };
  entries.sort((a, b) => {
    const orderA = filterOrder[a.filter] ?? 999;
    const orderB = filterOrder[b.filter] ?? 999;
    if (orderA !== orderB) return orderA - orderB;
    return a.lib.localeCompare(b.lib);
  });

  // Calculate comparisons
  // Filter -> Lib -> Ops
  const comparisons = {};
  for (const { lib, filter, size, metrics } of entries) {
    if (!comparisons[filter]) comparisons[filter] = [];
    if (metrics.contains?.mean_ns) {
      const ops = parseInt(size) / (metrics.contains.mean_ns / 1e9);
      comparisons[filter].push({ lib, ops });
    }
  }

  for (const { lib, filter, size, metrics } of entries) {
    const build_ops = metrics.build?.mean_ns
      ? (parseInt(size) / (metrics.build.mean_ns / 1e9) / 10000).toFixed(2)
      : "N/A";
    const query_ops = metrics.contains?.mean_ns
      ? (parseInt(size) / (metrics.contains.mean_ns / 1e9) / 10000).toFixed(2)
      : "N/A";
    const memory = formatBytes(metrics.memory?.bytes || 0);

    let speedup = "-";
    if (comparisons[filter] && comparisons[filter].length > 1) {
      // Compare against the library that is not "this" one.
      // If there are only 2, simple. If more, maybe compare to min?
      // Convention: compare to 'xorf' if current is 'jdb', or vice-versa?
      // Or just find the best relative to worst?
      // Let's compare current vs xorf if available, else vs slowest.
      // Actually standard is usually: if jdb is faster than xorf, show xX.

      // Let's start with: compare against the *other* one if exactly 2.
      const others = comparisons[filter].filter(c => c.lib !== lib);
      if (others.length === 1) {
        const other = others[0];
        const selfOps = comparisons[filter].find(c => c.lib === lib).ops;
        if (selfOps > other.ops) {
          speedup = `${(selfOps / other.ops).toFixed(2)}x`;
        }
      }
    }

    table += `| ${lib} | ${filter} | ${build_ops} | ${query_ops} | ${memory} | ${speedup} |\n`;
  }

  return table;
};

const displayPerfTable = (results, headers) => {
  const table = new Table({
    head: headers,
    chars: {
      top: "",
      "top-mid": "",
      "top-left": "",
      "top-right": "",
      bottom: "",
      "bottom-mid": "",
      "bottom-left": "",
      "bottom-right": "",
      left: "",
      "left-mid": "",
      mid: "",
      "mid-mid": "",
      right: "",
      "right-mid": "",
      middle: " ",
    },
    style: {
      head: ["cyan", "bold"],
      "padding-left": 1,
      "padding-right": 1,
    },
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

  const filterOrder = { BinaryFuse8: 0, BinaryFuse16: 1, BinaryFuse32: 2 };
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

  const filterOrder = { BinaryFuse8: 0, BinaryFuse16: 1, BinaryFuse32: 2 };
  entries.sort((a, b) => {
    const orderA = filterOrder[a.filter] ?? 999;
    const orderB = filterOrder[b.filter] ?? 999;
    if (orderA !== orderB) return orderA - orderB;
    return a.lib.localeCompare(b.lib);
  });

  for (const { lib, filter, metrics } of entries) {
    const fp_rate = (metrics.false_positive?.rate || 0).toFixed(5) + "%";
    const fn_rate = metrics.false_negative?.rate || 0;
    table += `| ${lib} | ${filter} | ${fp_rate} | ${fn_rate} |\n`;
  }

  return table;
};

const displayAccuracyTable = (results, headers) => {
  const table = new Table({
    head: headers,
    chars: {
      top: "",
      "top-mid": "",
      "top-left": "",
      "top-right": "",
      bottom: "",
      "bottom-mid": "",
      "bottom-left": "",
      "bottom-right": "",
      left: "",
      "left-mid": "",
      mid: "",
      "mid-mid": "",
      right: "",
      "right-mid": "",
      middle: " ",
    },
    style: {
      head: ["cyan", "bold"],
      "padding-left": 1,
      "padding-right": 1,
    },
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

  const filterOrder = { BinaryFuse8: 0, BinaryFuse16: 1, BinaryFuse32: 2 };
  entries.sort((a, b) => {
    const orderA = filterOrder[a.filter] ?? 999;
    const orderB = filterOrder[b.filter] ?? 999;
    if (orderA !== orderB) return orderA - orderB;
    return a.lib.localeCompare(b.lib);
  });

  for (const { lib, filter, metrics } of entries) {
    const fp_rate = (metrics.false_positive?.rate || 0).toFixed(5) + "%";
    const fn_rate = metrics.false_negative?.rate || 0;
    table.push([lib, filter, fp_rate, fn_rate]);
  }

  console.log(table.toString());
};

const main = async () => {
  const target_dir = await getTargetDir();
  const results = parseCriterion(target_dir);

  mkdirSync("readme", { recursive: true });

  await Promise.all(
    LANGS.map(({ code, headers, titles }) => {
      const perfTable = genPerfTable(results, headers.perf);
      const accuracyTable = genAccuracyTable(results, headers.accuracy);

      const content = `## ${titles.perf}\n\n${perfTable}\n## ${titles.accuracy}\n\n${accuracyTable}`;
      const path = `readme/${code}.bench.md`;
      writeFileSync(path, content);
      console.log(`  ${path}`);
    }),
  );

  // Display tables to console using cli-table3
  // Display tables to console using cli-table3
  const targetCode = LANG;
  const targetLang = LANGS.find((l) => l.code === targetCode);

  /*
   The console output needs to align with regress.js, so we should try to reuse getLabels if possible for generic terms,
   but table.js has specific column headers defined in LANGS structure.
   Ideally we merge them, but for now I will just use the LANGS structure as before but simplify the detection logic.
  */

  if (targetLang) {
    const { code, headers } = targetLang;
    console.log(`\n${code.toUpperCase()} - Performance Benchmark:`);
    const perfTable = new Table({
      head: headers.perf,
      chars: {
        top: "",
        "top-mid": "",
        "top-left": "",
        "top-right": "",
        bottom: "",
        "bottom-mid": "",
        "bottom-left": "",
        "bottom-right": "",
        left: "",
        "left-mid": "",
        mid: "",
        "mid-mid": "",
        right: "",
        "right-mid": "",
        middle: " ",
      },
      style: {
        head: ["cyan", "bold"],
        "padding-left": 0,
        "padding-right": 0,
      },
    });

    // ... populate perfTable similar to displayPerfTable logic but inline or refactored ...
    // Reusing logic block for simplicity
    const entries = [];
    for (const [lib, filters] of Object.entries(results)) {
      for (const [filter, sizes] of Object.entries(filters)) {
        for (const [size, metrics] of Object.entries(sizes)) {
          entries.push({ lib, filter, size, metrics });
        }
      }
    }
    const filterOrder = { BinaryFuse8: 0, BinaryFuse16: 1, BinaryFuse32: 2 };
    entries.sort((a, b) => {
      const orderA = filterOrder[a.filter] ?? 999;
      const orderB = filterOrder[b.filter] ?? 999;
      if (orderA !== orderB) return orderA - orderB;
      return a.lib.localeCompare(b.lib);
    });
    // Calculate comparisons for console table
    const comparisons = {};
    for (const { lib, filter, size, metrics } of entries) {
      if (!comparisons[filter]) comparisons[filter] = [];
      if (metrics.contains?.mean_ns) {
        const ops = parseInt(size) / (metrics.contains.mean_ns / 1e9);
        comparisons[filter].push({ lib, ops });
      }
    }

    for (const { lib, filter, size, metrics } of entries) {
      const build_ops = metrics.build?.mean_ns
        ? (parseInt(size) / (metrics.build.mean_ns / 1e9) / 10000).toFixed(2)
        : "N/A";
      const query_ops = metrics.contains?.mean_ns
        ? (parseInt(size) / (metrics.contains.mean_ns / 1e9) / 10000).toFixed(2)
        : "N/A";
      const memory = formatBytes(metrics.memory?.bytes || 0);

      let speedup = "-";
      if (comparisons[filter] && comparisons[filter].length > 1) {
        const others = comparisons[filter].filter(c => c.lib !== lib);
        if (others.length === 1) {
          const other = others[0];
          const selfOps = comparisons[filter].find(c => c.lib === lib).ops;
          if (selfOps > other.ops) {
            speedup = `${(selfOps / other.ops).toFixed(2)}x`;
          }
        }
      }
      perfTable.push([lib, filter, build_ops, query_ops, memory, speedup]);
    }
    console.log(perfTable.toString());

    console.log(`\n${code.toUpperCase()} - Accuracy:`);
    const accTable = new Table({
      head: headers.accuracy,
      chars: {
        top: "",
        "top-mid": "",
        "top-left": "",
        "top-right": "",
        bottom: "",
        "bottom-mid": "",
        "bottom-left": "",
        "bottom-right": "",
        left: "",
        "left-mid": "",
        mid: "",
        "mid-mid": "",
        right: "",
        "right-mid": "",
        middle: " ",
      },
      style: {
        head: ["cyan", "bold"],
        "padding-left": 0,
        "padding-right": 0,
      },
    });
    for (const { lib, filter, metrics } of entries) {
      const fp_rate = (metrics.false_positive?.rate || 0).toFixed(5) + "%";
      const fn_rate = metrics.false_negative?.rate || 0;
      accTable.push([lib, filter, fp_rate, fn_rate]);
    }
    console.log(accTable.toString());
  }
};

main();
