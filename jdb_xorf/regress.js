#!/usr/bin/env zx

import { readFileSync, writeFileSync, existsSync } from "fs";
import { getTargetDir, parseCriterion, compareFilters } from "./benches/lib.js";
import Table from "cli-table3";
import { $ } from "zx";

import LABELS, { LANG } from "./benches/i18n/index.js";

const MAX_HISTORY = 512;
const HISTORY_FILE = "benches/history.json";

const loadHistory = () => {
  if (!existsSync(HISTORY_FILE)) {
    return [];
  }
  return JSON.parse(readFileSync(HISTORY_FILE, "utf8"));
};

const saveHistory = (history) => {
  writeFileSync(HISTORY_FILE, JSON.stringify(history, null, 2));
};

const iterateResults = (results, callback) => {
  for (const [lib, filters] of Object.entries(results)) {
    for (const [filter, sizes] of Object.entries(filters)) {
      for (const [size, metrics] of Object.entries(sizes)) {
        callback(lib, filter, size, metrics);
      }
    }
  }
};

const formatValue = (value, defaultVal = "N/A") => {
  return value !== undefined && value !== null ? String(value) : defaultVal;
};

// Detect language
const langCode = LANG;

const formatTable = (current_results, previous_results = null) => {
  // 收集所有数据，按库分组
  const dataByLib = {};
  const sizes = new Set();

  iterateResults(current_results, (lib, filter, size, metrics) => {
    if (!dataByLib[lib]) {
      dataByLib[lib] = {};
    }
    if (!dataByLib[lib][filter]) {
      dataByLib[lib][filter] = {};
    }

    sizes.add(size);

    // 计算吞吐量 (万 ops/s)
    // build: mean_ns 是构建一次的时间，所以用 size / (mean_ns / 10^9) / 10000
    const buildThroughput = metrics.build?.mean_ns
      ? (parseInt(size) / (metrics.build.mean_ns / 1000000000) / 10000).toFixed(
        2,
      )
      : "N/A";
    // query: mean_ns 是整个迭代的时间（包含所有 size 次查询），所以用 size / (mean_ns / 10^9) / 10000
    const queryThroughput = metrics.has?.mean_ns
      ? (
        parseInt(size) /
        (metrics.has.mean_ns / 1000000000) /
        10000
      ).toFixed(2)
      : "N/A";

    dataByLib[lib][filter][size] = {
      build_ops: buildThroughput,
      query_ops: queryThroughput,
      build_ns: metrics.build?.mean_ns,
      query_ns: metrics.has?.mean_ns,
      memory_kb: metrics.memory?.bytes
        ? (metrics.memory.bytes / 1024).toFixed(1)
        : "N/A",
      fp_rate:
        metrics.false_positive?.rate !== undefined
          ? metrics.false_positive.rate.toFixed(3)
          : "N/A",
    };
  });

  if (Object.keys(dataByLib).length === 0) return;

  // 按大小排序
  const sortedSizes = Array.from(sizes).sort(
    (a, b) => parseInt(a) - parseInt(b),
  );
  const sortedLibs = Object.keys(dataByLib).sort();

  for (const lib of sortedLibs) {
    // 库标题行
    console.log(`→ ${lib}\n`);

    const sortedFilters = Object.keys(dataByLib[lib]).sort((a, b) => {
      return compareFilters(a, b);
    });

    // 构建列名（过滤器+大小）
    const colKeys = [];
    for (const filter of sortedFilters) {
      for (const size of sortedSizes) {
        const data = dataByLib[lib][filter][size];
        if (data) {
          colKeys.push({ filter, size, data });
        }
      }
    }

    // 构建表头
    const head = [LABELS.metric];
    const colWidths = [15];
    for (const col of colKeys) {
      head.push(`${col.filter}\n${col.size}`);
      colWidths.push(20);
    }

    const table = new Table({
      head,
      colWidths,
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

    // 构建行（每行一个指标）
    const metrics = [
      {
        name: LABELS.build_ops,
        key: "build_ops",
        isThroughput: true,
        isBf: true,
      },
      {
        name: LABELS.query_ops,
        key: "query_ops",
        isThroughput: true,
        isBf: false,
      },
      { name: LABELS.memory_kb, key: "memory_kb", isThroughput: false },
      { name: LABELS.fp_rate, key: "fp_rate", isThroughput: false },
    ];

    for (const metric of metrics) {
      const row = [metric.name];

      for (const col of colKeys) {
        let valueStr = formatValue(col.data[metric.key]);

        // 如果有历史数据且是吞吐量指标，显示变化
        if (
          previous_results &&
          metric.isThroughput &&
          col.data.build_ns &&
          col.data.query_ns
        ) {
          const prev = previous_results[lib]?.[col.filter]?.[col.size];
          if (prev) {
            if (metric.isBf && prev.build?.mean_ns) {
              const prevVal = (
                parseInt(col.size) /
                (prev.build.mean_ns / 1000000000) /
                10000
              ).toFixed(2);
              const currVal = parseFloat(col.data.build_ops);
              const prevNum = parseFloat(prevVal);
              if (prevNum > 0) {
                const change = (((currVal - prevNum) / prevNum) * 100).toFixed(
                  2,
                );
                if (Math.abs(change) > 0.01) {
                  valueStr += ` (${change >= 0 ? "+" : ""}${change}%)`;
                }
              }
            } else if (!metric.isBf && prev.has?.mean_ns) {
              const prevVal = (
                parseInt(col.size) /
                (prev.has.mean_ns / 1000000000) /
                10000
              ).toFixed(2);
              const currVal = parseFloat(col.data.query_ops);
              const prevNum = parseFloat(prevVal);
              if (prevNum > 0) {
                const change = (((currVal - prevNum) / prevNum) * 100).toFixed(
                  2,
                );
                if (Math.abs(change) > 0.01) {
                  valueStr += ` (${change >= 0 ? "+" : ""}${change}%)`;
                }
              }
            }
          }
        }

        row.push(valueStr);
      }

      table.push(row);
    }

    console.log(table.toString());
  }
};

const main = async () => {
  const target_dir = await getTargetDir();

  // Clean up all criterion data to avoid stale benchmarks showing up
  // 清理所有 criterion 数据以避免显示陈旧的基准测试
  await $`rm -rf ${target_dir}/criterion`.quiet();
  await $`rm -rf target/criterion`.quiet();

  // Run benchmarks with bench-jdb feature
  // 运行带 bench-jdb 特性的性能测试
  console.log(LABELS.running_bench);
  await $`cargo bench --bench bench --features bench-jdb`.quiet();

  const current_results = parseCriterion(target_dir);

  const history = loadHistory();
  const previous =
    history.length > 0 ? history[history.length - 1].results : null;

  formatTable(current_results, previous);

  history.push({
    timestamp: new Date().toISOString(),
    results: current_results,
  });

  if (history.length > MAX_HISTORY) {
    history.splice(0, history.length - MAX_HISTORY);
  }

  saveHistory(history);
  console.log(`${LABELS.saved_history} (${history.length}/${MAX_HISTORY})`);
};

main();
