#!/usr/bin/env bun

// Read bench.json and generate markdown tables with system info
// 读取 bench.json 并生成包含系统信息的 markdown 表格

import { readFileSync, writeFileSync } from "fs";
import { join } from "path";
import { execSync } from "child_process";
import os from "os";
import Table from "cli-table3";

const ROOT = import.meta.dirname;
const JSON_PATH = join(ROOT, "bench.json");
const EN_MD = join(ROOT, "readme/en.bench.md");
const ZH_MD = join(ROOT, "readme/zh.bench.md");

const data = JSON.parse(readFileSync(JSON_PATH, "utf8"));
const base = data.results[0];

// Get system info
// 获取系统信息
const sysInfo = getSystemInfo();

// Console output with cli-table3
// 使用 cli-table3 输出到控制台
console.log(
  `\n=== Performance (${data.items} items, capacity=${data.capacity}) ===\n`,
);
printConsoleTable(data);

// Generate markdown files
// 生成 markdown 文件
writeFileSync(EN_MD, genMdEn(data, sysInfo));
writeFileSync(ZH_MD, genMdZh(data, sysInfo));

function printConsoleTable(data) {
  const table = new Table({
    head: [
      "Library",
      "FPP",
      "Contains (M/s)",
      "Add (M/s)",
      "Remove (M/s)",
      "Memory (KB)",
    ],
    style: { head: ["cyan"] },
  });

  for (const r of data.results) {
    const ar = (r.add_mops / base.add_mops).toFixed(2);
    const cr = (r.contains_mops / base.contains_mops).toFixed(2);
    const rr = (r.remove_mops / base.remove_mops).toFixed(2);
    table.push([
      r.lib,
      `${(r.fpp * 100).toFixed(2)}%`,
      `${r.contains_mops.toFixed(2)} (${cr})`,
      `${r.add_mops.toFixed(2)} (${ar})`,
      `${r.remove_mops.toFixed(2)} (${rr})`,
      r.memory_kb.toFixed(1),
    ]);
  }

  console.log(table.toString());
}

function getSystemInfo() {
  const cpus = os.cpus();
  const cpu = cpus[0]?.model || "Unknown";
  const cores = cpus.length;
  const mem = (os.totalmem() / 1024 / 1024 / 1024).toFixed(1);
  const platform = os.platform();
  const arch = os.arch();
  const release = os.release();

  // Get Rust version
  // 获取 Rust 版本
  let rustVer = "Unknown";
  try {
    rustVer = execSync("rustc --version", { encoding: "utf8" }).trim();
  } catch {}

  // Get OS name (macOS specific)
  // 获取操作系统名称（macOS 特定）
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
}

function genMdEn(data, sys) {
  const lines = [];

  lines.push("## Benchmark Results\n");

  // System info
  // 系统信息
  lines.push("### Test Environment\n");
  lines.push(`| Item | Value |`);
  lines.push(`|------|-------|`);
  lines.push(`| OS | ${sys.osName} (${sys.arch}) |`);
  lines.push(`| CPU | ${sys.cpu} |`);
  lines.push(`| Cores | ${sys.cores} |`);
  lines.push(`| Memory | ${sys.mem} GB |`);
  lines.push(`| Rust | ${sys.rustVer} |`);
  lines.push("");

  lines.push(
    `Test: ${data.items} items, capacity=${data.capacity}\n`,
  );

  // FPP explanation
  // FPP 说明
  lines.push("### What is FPP?\n");
  lines.push(
    '**FPP (False Positive Probability)** is the probability that a filter incorrectly reports an item as present when it was never added. Lower FPP means higher accuracy but requires more memory. A typical FPP of 1% means about 1 in 100 queries for non-existent items will incorrectly return "possibly exists".\n',
  );

  // Performance table
  // 性能表格
  lines.push("### Performance Comparison\n");
  lines.push(
    "| Library | FPP | Contains (M/s) | Add (M/s) | Remove (M/s) | Memory (KB) |",
  );
  lines.push(
    "|---------|-----|----------------|-----------|--------------|-------------|",
  );

  for (const r of data.results) {
    const ar = (r.add_mops / base.add_mops).toFixed(2);
    const cr = (r.contains_mops / base.contains_mops).toFixed(2);
    const rr = (r.remove_mops / base.remove_mops).toFixed(2);
    lines.push(
      `| ${r.lib} | ${(r.fpp * 100).toFixed(2)}% | ${r.contains_mops.toFixed(2)} (${cr}) | ${r.add_mops.toFixed(2)} (${ar}) | ${r.remove_mops.toFixed(2)} (${rr}) | ${r.memory_kb.toFixed(1)} |`,
    );
  }

  lines.push(
    "\n*Ratio in parentheses: relative to autoscale_cuckoo_filter (1.00 = baseline)*",
  );

  return lines.join("\n");
}

function genMdZh(data, sys) {
  const lines = [];

  lines.push("## 性能测试结果\n");

  // System info
  // 系统信息
  lines.push("### 测试环境\n");
  lines.push(`| 项目 | 值 |`);
  lines.push(`|------|-------|`);
  lines.push(`| 操作系统 | ${sys.osName} (${sys.arch}) |`);
  lines.push(`| CPU | ${sys.cpu} |`);
  lines.push(`| 核心数 | ${sys.cores} |`);
  lines.push(`| 内存 | ${sys.mem} GB |`);
  lines.push(`| Rust | ${sys.rustVer} |`);
  lines.push("");

  lines.push(
    `测试：${data.items} 条数据，容量=${data.capacity}\n`,
  );

  // FPP explanation
  // 误判率说明
  lines.push("### 什么是误判率（FPP）？\n");
  lines.push(
    "**误判率（False Positive Probability，FPP）** 是指过滤器错误地报告某个元素存在的概率，即该元素实际上从未被添加过。误判率越低，准确性越高，但需要更多内存。典型的 1% 误判率意味着大约每 100 次查询不存在的元素，会有 1 次错误地返回「可能存在」。\n",
  );

  // Performance table
  // 性能表格
  lines.push("### 性能对比\n");
  lines.push(
    "| 库 | 误判率 | 查询 (百万/秒) | 添加 (百万/秒) | 删除 (百万/秒) | 内存 (KB) |",
  );
  lines.push(
    "|---------|-----|----------------|-----------|--------------|-------------|",
  );

  for (const r of data.results) {
    const ar = (r.add_mops / base.add_mops).toFixed(2);
    const cr = (r.contains_mops / base.contains_mops).toFixed(2);
    const rr = (r.remove_mops / base.remove_mops).toFixed(2);
    lines.push(
      `| ${r.lib} | ${(r.fpp * 100).toFixed(2)}% | ${r.contains_mops.toFixed(2)} (${cr}) | ${r.add_mops.toFixed(2)} (${ar}) | ${r.remove_mops.toFixed(2)} (${rr}) | ${r.memory_kb.toFixed(1)} |`,
    );
  }

  lines.push(
    "\n*括号内为相对性能：以 autoscale_cuckoo_filter 为基准（1.00 = 基准值）*",
  );

  return lines.join("\n");
}
