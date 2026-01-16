#!/usr/bin/env bun

import { readFileSync, writeFileSync } from "fs";
import { join } from "path";
import { execSync } from "child_process";
import os from "os";
import Table from "cli-table3";
import {
  ALGORITHM_NAMES,
  ALGORITHM_NAMES_ZH,
  EPSILON_EXPLANATIONS,
  fmtTime,
  fmtThroughput,
  groupByDataSize,
  getSortedDataSizes,
  formatDataSize,
  formatMemory,
} from "./js/common.js";

const ROOT = import.meta.dirname;
const JSON_PATH = join(ROOT, "bench.json");
const ACCURACY_PATH = join(ROOT, "accuracy.json");
const BUILD_TIME_PATH = join(ROOT, "build_time.json");
const EN_MD = join(ROOT, "readme/en.bench.md");
const ZH_MD = join(ROOT, "readme/zh.bench.md");

const getSystemInfo = () => {
  const cpus = os.cpus();
  const cpu = cpus[0]?.model || "Unknown";
  const cores = cpus.length;
  const mem = (os.totalmem() / 1024 / 1024 / 1024).toFixed(1);
  const platform = os.platform();
  const arch = os.arch();
  const release = os.release();

  let rustVer = "Unknown";
  try {
    rustVer = execSync("rustc --version", { encoding: "utf8" }).trim();
  } catch {}

  let osName = `${platform} ${release}`;
  if (platform === "darwin") {
    try {
      const ver = execSync("sw_vers -productVersion", {
        encoding: "utf8",
      }).trim();
      osName = `macOS ${ver}`;
    } catch {}
  }

  return { cpu, cores, mem, osName, arch, rustVer };
};

const printConfig = (config) => {
  console.log(`Benchmark Configuration:
  Query Count: ${config.query_count}
  Data Sizes: ${config.data_sizes.join(", ")}
  Epsilon Values: ${config.epsilon_values.join(", ")}`);
};

const printConsoleTable = (results, accuracyData, buildTimeData) => {
  const grouped = groupByDataSize(results);

  // Print performance tables for all data sizes
  for (const [dataSize, groupResults] of getSortedDataSizes(grouped)) {
    const table = new Table({
      head: ["Algorithm", "Mean Time", "Std Dev", "Throughput", "Memory"],
      style: { head: ["cyan"] },
    });

    for (const r of groupResults.sort((a, b) => b.throughput - a.throughput)) {
      const epsilonStr = r.epsilon !== undefined ? ` (ε=${r.epsilon})` : "";
      const memStr = r.memory_bytes > 0 ? formatMemory(r.memory_bytes) : "-";
      table.push([
        ALGORITHM_NAMES[r.algorithm] + epsilonStr,
        fmtTime(r.mean_ns),
        fmtTime(r.std_dev_ns),
        fmtThroughput(r.throughput),
        memStr,
      ]);
    }

    console.log(`\nData Size: ${formatDataSize(dataSize)}`);
    console.log(table.toString());
  }

  // Print accuracy table
  console.log("\n" + "=".repeat(80));
  console.log("Accuracy Comparison: jdb_pgm vs pgm_index");
  console.log("=".repeat(80));

  const accuracyGrouped = {};
  for (const r of accuracyData.results) {
    const key = `${r.data_size}_eps_${r.epsilon}`;
    if (!accuracyGrouped[key]) {
      accuracyGrouped[key] = {
        data_size: r.data_size,
        epsilon: r.epsilon,
        jdb_pgm: null,
        external_pgm: null,
      };
    }
    accuracyGrouped[key][r.algorithm] = r;
  }

  const accTable = new Table({
    head: [
      "Data Size",
      "Epsilon",
      "jdb_pgm (Max)",
      "jdb_pgm (Avg)",
      "pgm_index (Max)",
      "pgm_index (Avg)",
    ],
    style: { head: ["cyan"] },
  });

  const sortedAccKeys = Object.keys(accuracyGrouped).sort();
  for (const key of sortedAccKeys) {
    const { data_size, epsilon, jdb_pgm, external_pgm } = accuracyGrouped[key];
    const jdbMaxError = jdb_pgm?.max_error ?? "N/A";
    const jdbAvgError = jdb_pgm?.avg_error?.toFixed(2) ?? "N/A";
    const extMaxError = external_pgm?.max_error ?? "N/A";
    const extAvgError = external_pgm?.avg_error?.toFixed(2) ?? "N/A";
    accTable.push([
      formatDataSize(data_size),
      epsilon,
      jdbMaxError,
      jdbAvgError,
      extMaxError,
      extAvgError,
    ]);
  }
  console.log(accTable.toString());

  // Print build time table
  console.log("\n" + "=".repeat(80));
  console.log("Build Time Comparison: jdb_pgm vs pgm_index");
  console.log("=".repeat(80));

  const buildTimeGrouped = {};
  for (const r of buildTimeData.results) {
    const key = `${r.data_size}_eps_${r.epsilon}`;
    if (!buildTimeGrouped[key]) {
      buildTimeGrouped[key] = {
        data_size: r.data_size,
        epsilon: r.epsilon,
        jdb_pgm: null,
        external_pgm: null,
      };
    }
    buildTimeGrouped[key][r.algorithm] = r;
  }

  const buildTable = new Table({
    head: [
      "Data Size",
      "Epsilon",
      "jdb_pgm (Time)",
      "pgm_index (Time)",
      "Speedup",
    ],
    style: { head: ["cyan"] },
  });

  const sortedBuildKeys = Object.keys(buildTimeGrouped).sort();
  for (const key of sortedBuildKeys) {
    const { data_size, epsilon, jdb_pgm, external_pgm } = buildTimeGrouped[key];
    const jdbTime = fmtTime(jdb_pgm?.build_time_ns || 0);
    const extTime = fmtTime(external_pgm?.build_time_ns || 0);
    const speedup =
      jdb_pgm && external_pgm && external_pgm.build_time_ns > 0
        ? (external_pgm.build_time_ns / jdb_pgm.build_time_ns).toFixed(2) + "x"
        : "N/A";
    buildTable.push([
      formatDataSize(data_size),
      epsilon,
      jdbTime,
      extTime,
      speedup,
    ]);
  }
  console.log(buildTable.toString());
};

const genMdEn = (data, sys, accuracyData, buildTimeData) => {
  const { config, results } = data;
  const grouped = groupByDataSize(results);

  // Only include 1,000,000 data size
  let tables = "";
  for (const [dataSize, groupResults] of getSortedDataSizes(grouped)) {
    if (parseInt(dataSize) !== 1000000) continue; // Skip all except 1,000,000

    const rows = groupResults
      .sort((a, b) => b.throughput - a.throughput)
      .map((r) => {
        const epsilon = r.epsilon !== undefined ? r.epsilon : "N/A";
        const memStr = r.memory_bytes > 0 ? formatMemory(r.memory_bytes) : "-";
        return `| ${ALGORITHM_NAMES[r.algorithm]} | ${epsilon} | ${fmtTime(r.mean_ns)} | ${fmtTime(r.std_dev_ns)} | ${fmtThroughput(r.throughput)} | ${memStr} |`;
      })
      .join("\n");

    tables += `### Data Size: ${formatDataSize(dataSize)}

| Algorithm | Epsilon | Mean Time | Std Dev | Throughput | Memory |
|-----------|---------|-----------|---------|------------|--------|
${rows}

`;
  }

  const accuracyTable = genAccuracyTableEn(accuracyData);
  const buildTimeTable = genBuildTimeTableEn(buildTimeData);

  const { title, definition, example, examples } = EPSILON_EXPLANATIONS.en;
  const epsilonSection = `

---

### Epsilon (ε) Explained

${title}

${definition}

${example}
${examples.map((item) => `- ${item}`).join("\n")}
`;

  return `## Pgm-Index Benchmark

Performance comparison of Pgm-Index vs Binary Search with different epsilon values.

${tables}${accuracyTable}${buildTimeTable}### Configuration
Query Count: ${config.query_count}
Data Sizes: ${config.data_sizes.map((s) => s.toLocaleString()).join(", ")}
Epsilon Values: ${config.epsilon_values.join(", ")}

${epsilonSection}

### Notes
#### What is Pgm-Index?
Pgm-Index (Piecewise Geometric Model Index) is a learned index structure that approximates the distribution of keys with piecewise linear models.
It provides O(log ε) search time with guaranteed error bounds, where ε controls the trade-off between memory and speed.

#### Why Compare with Binary Search?
Binary search is the baseline for sorted array lookup. Pgm-Index aims to:
- Match or exceed binary search performance
- Reduce memory overhead compared to traditional indexes
- Provide better cache locality for large datasets

#### Environment
- OS: ${sys.osName} (${sys.arch})
- CPU: ${sys.cpu}
- Cores: ${sys.cores}
- Memory: ${sys.mem}GB
- Rust: ${sys.rustVer}

#### References
- [Pgm-Index Paper](https://doi.org/10.1145/3373718.3394764)
- [Official Pgm-Index Site](https://pgm.di.unipi.it/)
- [Learned Indexes](https://arxiv.org/abs/1712.01208)
`;
};

const genAccuracyTableEn = (accuracyData) => {
  const { results } = accuracyData;
  const grouped = {};

  for (const r of results) {
    const key = `${r.data_size}_eps_${r.epsilon}`;
    if (!grouped[key]) {
      grouped[key] = {
        data_size: r.data_size,
        epsilon: r.epsilon,
        jdb_pgm: null,
        external_pgm: null,
      };
    }
    grouped[key][r.algorithm] = r;
  }

  let table = "### Accuracy Comparison: jdb_pgm vs pgm_index\n\n";
  table +=
    "| Data Size | Epsilon | jdb_pgm (Max) | jdb_pgm (Avg) | pgm_index (Max) | pgm_index (Avg) |\n";
  table +=
    "|-----------|---------|---------------|---------------|-----------------|------------------|\n";

  const sortedKeys = Object.keys(grouped)
    .sort()
    .filter((key) => {
      const { data_size } = grouped[key];
      return parseInt(data_size) === 1000000;
    });

  for (const key of sortedKeys) {
    const { data_size, epsilon, jdb_pgm, external_pgm } = grouped[key];
    const jdbMax = jdb_pgm?.max_error ?? "N/A";
    const jdbAvg = jdb_pgm?.avg_error?.toFixed(2) ?? "N/A";
    const extMax = external_pgm?.max_error ?? "N/A";
    const extAvg = external_pgm?.avg_error?.toFixed(2) ?? "N/A";
    table += `| ${formatDataSize(data_size)} | ${epsilon} | ${jdbMax} | ${jdbAvg} | ${extMax} | ${extAvg} |\n`;
  }

  return table;
};

const genBuildTimeTableEn = (buildTimeData) => {
  const { results } = buildTimeData;
  const grouped = {};

  // Group by data_size and epsilon
  for (const r of results) {
    const key = `${r.data_size}_eps_${r.epsilon}`;
    if (!grouped[key]) {
      grouped[key] = {
        data_size: r.data_size,
        epsilon: r.epsilon,
        jdb_pgm: null,
        external_pgm: null,
      };
    }
    grouped[key][r.algorithm] = r;
  }

  let table = "### Build Time Comparison: jdb_pgm vs pgm_index\n\n";
  table +=
    "| Data Size | Epsilon | jdb_pgm (Time) | pgm_index (Time) | Speedup |\n";
  table +=
    "|-----------|---------|---------------------|-----------------|---------|\n";

  // Only include 1,000,000 data size
  const sortedKeys = Object.keys(grouped)
    .sort()
    .filter((key) => {
      const { data_size } = grouped[key];
      return parseInt(data_size) === 1000000;
    });

  for (const key of sortedKeys) {
    const { data_size, epsilon, jdb_pgm, external_pgm } = grouped[key];
    const jdbTime = fmtTime(jdb_pgm?.build_time_ns || 0);
    const extTime = fmtTime(external_pgm?.build_time_ns || 0);
    const speedup =
      jdb_pgm && external_pgm && external_pgm.build_time_ns > 0
        ? (external_pgm.build_time_ns / jdb_pgm.build_time_ns).toFixed(2) + "x"
        : "N/A";
    table += `| ${formatDataSize(data_size)} | ${epsilon} | ${jdbTime} | ${extTime} | ${speedup} |\n`;
  }

  return table;
};

const genAccuracyTableZh = (accuracyData) => {
  const { results } = accuracyData;
  const grouped = {};

  for (const r of results) {
    const key = `${r.data_size}_eps_${r.epsilon}`;
    if (!grouped[key]) {
      grouped[key] = {
        data_size: r.data_size,
        epsilon: r.epsilon,
        jdb_pgm: null,
        external_pgm: null,
      };
    }
    grouped[key][r.algorithm] = r;
  }

  let table = "### 精度对比: jdb_pgm vs pgm_index\n\n";
  table +=
    "| 数据大小 | Epsilon | jdb_pgm (最大) | jdb_pgm (平均) | pgm_index (最大) | pgm_index (平均) |\n";
  table +=
    "|----------|---------|----------------|----------------|------------------|-------------------|\n";

  const sortedKeys = Object.keys(grouped)
    .sort()
    .filter((key) => {
      const { data_size } = grouped[key];
      return parseInt(data_size) === 1000000;
    });

  for (const key of sortedKeys) {
    const { data_size, epsilon, jdb_pgm, external_pgm } = grouped[key];
    const jdbMax = jdb_pgm?.max_error ?? "N/A";
    const jdbAvg = jdb_pgm?.avg_error?.toFixed(2) ?? "N/A";
    const extMax = external_pgm?.max_error ?? "N/A";
    const extAvg = external_pgm?.avg_error?.toFixed(2) ?? "N/A";
    table += `| ${formatDataSize(data_size)} | ${epsilon} | ${jdbMax} | ${jdbAvg} | ${extMax} | ${extAvg} |\n`;
  }

  return table;
};

const genBuildTimeTableZh = (buildTimeData) => {
  const { results } = buildTimeData;
  const grouped = {};

  // Group by data_size and epsilon
  for (const r of results) {
    const key = `${r.data_size}_eps_${r.epsilon}`;
    if (!grouped[key]) {
      grouped[key] = {
        data_size: r.data_size,
        epsilon: r.epsilon,
        jdb_pgm: null,
        external_pgm: null,
      };
    }
    grouped[key][r.algorithm] = r;
  }

  let table = "### 构建时间对比: jdb_pgm vs pgm_index\n\n";
  table +=
    "| 数据大小 | Epsilon | jdb_pgm (时间) | pgm_index (时间) | 加速比 |\n";
  table +=
    "|----------|---------|---------------------|-----------------|--------|\n";

  // Only include 1,000,000 data size
  const sortedKeys = Object.keys(grouped)
    .sort()
    .filter((key) => {
      const { data_size } = grouped[key];
      return parseInt(data_size) === 1000000;
    });

  for (const key of sortedKeys) {
    const { data_size, epsilon, jdb_pgm, external_pgm } = grouped[key];
    const jdbTime = fmtTime(jdb_pgm?.build_time_ns || 0);
    const extTime = fmtTime(external_pgm?.build_time_ns || 0);
    const speedup =
      jdb_pgm && external_pgm && external_pgm.build_time_ns > 0
        ? (external_pgm.build_time_ns / jdb_pgm.build_time_ns).toFixed(2) + "x"
        : "N/A";
    table += `| ${formatDataSize(data_size)} | ${epsilon} | ${jdbTime} | ${extTime} | ${speedup} |\n`;
  }

  return table;
};

const genMdZh = (data, sys, accuracyData, buildTimeData) => {
  const { config, results } = data;
  const grouped = groupByDataSize(results);

  // Only include 1,000,000 data size
  let tables = "";
  for (const [dataSize, groupResults] of getSortedDataSizes(grouped)) {
    if (parseInt(dataSize) !== 1000000) continue; // Skip all except 1,000,000

    const rows = groupResults
      .sort((a, b) => b.throughput - a.throughput)
      .map((r) => {
        const epsilon = r.epsilon !== undefined ? r.epsilon : "N/A";
        const memStr = r.memory_bytes > 0 ? formatMemory(r.memory_bytes) : "-";
        return `| ${ALGORITHM_NAMES_ZH[r.algorithm]} | ${epsilon} | ${fmtTime(r.mean_ns)} | ${fmtTime(r.std_dev_ns)} | ${fmtThroughput(r.throughput)} | ${memStr} |`;
      })
      .join("\n");

    tables += `### 数据大小: ${formatDataSize(dataSize)}

| 算法 | Epsilon | 平均时间 | 标准差 | 吞吐量 | 内存 |
|------|---------|----------|--------|--------|------|
${rows}

`;
  }

  const accuracyTable = genAccuracyTableZh(accuracyData);
  const buildTimeTable = genBuildTimeTableZh(buildTimeData);

  const { title, definition, example, examples } = EPSILON_EXPLANATIONS.zh;
  const epsilonSection = `

---

### Epsilon (ε) 说明

${title}

${definition}

${example}
${examples.map((item) => `- ${item}`).join("\n")}
`;

  return `## Pgm 索引评测

Pgm-Index 与二分查找在不同 epsilon 值下的性能对比。

${tables}${accuracyTable}${buildTimeTable}### 配置
查询次数: ${config.query_count}
数据大小: ${config.data_sizes.map((s) => s.toLocaleString()).join(", ")}
Epsilon 值: ${config.epsilon_values.join(", ")}

${epsilonSection}

### 备注
#### 什么是 Pgm-Index?
Pgm-Index（分段几何模型索引）是一种学习型索引结构，使用分段线性模型近似键的分布。
它提供 O(log ε) 的搜索时间，并保证误差边界，其中 ε 控制内存和速度之间的权衡。

#### 为什么与二分查找对比?
二分查找是已排序数组查找的基准。Pgm-Index 旨在：
- 匹配或超过二分查找的性能
- 相比传统索引减少内存开销
- 为大数据集提供更好的缓存局部性

#### 环境
- 系统: ${sys.osName} (${sys.arch})
- CPU: ${sys.cpu}
- 核心数: ${sys.cores}
- 内存: ${sys.mem}GB
- Rust版本: ${sys.rustVer}

#### 参考
- [Pgm-Index 论文](https://doi.org/10.1145/3373718.3394764)
- [Pgm-Index 官方网站](https://pgm.di.unipi.it/)
- [学习型索引](https://arxiv.org/abs/1712.01208)
`;
};

const main = () => {
  const data = JSON.parse(readFileSync(JSON_PATH, "utf8"));
  const accuracyData = JSON.parse(readFileSync(ACCURACY_PATH, "utf8"));
  const buildTimeData = JSON.parse(readFileSync(BUILD_TIME_PATH, "utf8"));
  const sys = getSystemInfo();

  printConfig(data.config);
  printConsoleTable(data.results, accuracyData, buildTimeData);

  const enMd = genMdEn(data, sys, accuracyData, buildTimeData);
  writeFileSync(EN_MD, enMd);
  console.log(`\nWritten: ${EN_MD}`);

  const zhMd = genMdZh(data, sys, accuracyData, buildTimeData);
  writeFileSync(ZH_MD, zhMd);
  console.log(`Written: ${ZH_MD}`);
};

main();
