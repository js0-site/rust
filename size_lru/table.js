#!/usr/bin/env bun

import { readFileSync, writeFileSync } from "fs";
import { join } from "path";
import { execSync } from "child_process";
import os from "os";
import Table from "cli-table3";

const ROOT = import.meta.dirname;
const JSON_PATH = join(ROOT, "bench.json");
const EN_MD = join(ROOT, "readme/en.bench.md");
const ZH_MD = join(ROOT, "readme/zh.bench.md");

const VALUE_TIERS = [
  { name: "Tiny Metadata", range: "16-100B", items_pct: 40, size_pct: 0.3, desc: "USR pool" },
  { name: "Small Structs", range: "100B-1KB", items_pct: 35, size_pct: 2.2, desc: "APP pool" },
  { name: "Medium Content", range: "1-10KB", items_pct: 20, size_pct: 12, desc: "Bandwidth test" },
  { name: "Large Objects", range: "10-100KB", items_pct: 4, size_pct: 24, desc: "VAR pool" },
  { name: "Huge Blobs", range: "100KB-1MB", items_pct: 1, size_pct: 61, desc: "Rare large" },
];

const OP_DIST = [
  { op: "Read", pct: 90, desc: "Cache lookups dominate", source: "Twitter: 99%+ reads, TAO: 99.8% reads" },
  { op: "Write", pct: 9, desc: "Updates/inserts", source: "TAO: ~0.1% writes, relaxed for testing" },
  { op: "Delete", pct: 1, desc: "Invalidations", source: "TAO: ~0.1% deletes" },
];

const fmtSize = (bytes) => {
  if (bytes >= 1024 * 1024) return `${(bytes / 1024 / 1024).toFixed(1)}MB`;
  if (bytes >= 1024) return `${(bytes / 1024).toFixed(1)}KB`;
  return `${bytes}B`;
};

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
      const ver = execSync("sw_vers -productVersion", { encoding: "utf8" }).trim();
      osName = `macOS ${ver}`;
    } catch {}
  }

  return { cpu, cores, mem, osName, arch, rustVer };
};

const printConfig = (cfg) => {
  console.log(`Benchmark Configuration:
  Memory Budget: ${fmtSize(cfg.mem_budget)}
  Operations: Read ${cfg.read_ratio}%, Write ${cfg.write_ratio}%, Delete ${cfg.delete_ratio}%
  Real Miss Rate: ${cfg.real_miss_ratio}%
  Zipf Exponent: ${cfg.zipf_s}
  Ops/Loop: ${cfg.ops_per_loop.toLocaleString()}, Loops: ${cfg.loops}`);
};

const printStats = (stats) => {
  console.log(`
Dataset Statistics:
  Items: ${stats.item_count.toLocaleString()}, Total: ${(stats.total_size_bytes / 1024).toFixed(1)}KB
  Item Size: avg=${stats.avg_item_size}B, min=${stats.min_item_size}B, max=${stats.max_item_size}B
  Memory Budget: ${(stats.mem_budget / 1024).toFixed(1)}KB`);
};

const printSizeDistribution = (dist) => {
  const table = new Table({
    head: ["Size Range", "Items", "Items %", "Total Size", "Size %"],
    style: { head: ["cyan"] },
  });
  for (const b of dist) {
    if (b.count > 0) {
      const sizeStr = fmtSize(b.total_size_bytes);
      table.push([b.label, b.count.toLocaleString(), `${b.percent.toFixed(1)}%`, sizeStr, `${b.size_percent.toFixed(1)}%`]);
    }
  }
  console.log(`
Size Distribution:
${table.toString()}`);
};

const printConsoleTable = (results) => {
  const best = results[0];
  const table = new Table({
    head: ["Library", "Hit Rate", "Effective OPS (M/s)", "Perf %", "Memory (KB)"],
    style: { head: ["cyan"] },
  });

  for (const r of results) {
    const perfPct = ((r.effective_ops / best.effective_ops) * 100).toFixed(0);
    table.push([
      r.lib,
      `${(r.hit_rate * 100).toFixed(2)}%`,
      `${(r.effective_ops / 1e6).toFixed(2)}`,
      `${perfPct}%`,
      r.memory_kb.toFixed(1),
    ]);
  }

  console.log(`
Benchmark Results:
${table.toString()}`);
};

const genMdEn = (data, sys) => {
  const { config: cfg, stats } = data;
  const lines = [];

  lines.push("## LRU Cache Benchmark\n");
  lines.push("Real-world data distribution, fixed memory budget, comparing hit rate and effective OPS.\n");

  const best = data.results[0];
  lines.push("### Results\n");
  lines.push("| Library | Hit Rate | Effective OPS | Perf | Memory |");
  lines.push("|---------|----------|---------------|------|--------|");

  for (const r of data.results) {
    const perfPct = ((r.effective_ops / best.effective_ops) * 100).toFixed(0);
    lines.push(
      `| ${r.lib} | ${(r.hit_rate * 100).toFixed(2)}% | ${(r.effective_ops / 1e6).toFixed(2)}M/s | ${perfPct}% | ${r.memory_kb.toFixed(1)}KB |`
    );
  }
  lines.push("");

  lines.push("### Configuration\n");
  lines.push(`Memory: ${fmtSize(cfg.mem_budget)} · Zipf s=${cfg.zipf_s} · R/W/D: ${cfg.read_ratio}/${cfg.write_ratio}/${cfg.delete_ratio}% · Miss: ${cfg.real_miss_ratio}% · Ops: ${(cfg.ops_per_loop / 1e6).toFixed(0)}M×${cfg.loops}\n`);

  lines.push("### Size Distribution\n");
  lines.push("| Range | Items | Size |");
  lines.push("|-------|-------|------|");
  for (const b of stats.size_distribution) {
    if (b.count > 0) {
      lines.push(`| ${b.label} | ${b.percent.toFixed(2)}% | ${b.size_percent.toFixed(2)}% |`);
    }
  }
  lines.push("");

  lines.push("---\n");
  lines.push("### Notes\n");
  lines.push("#### Data Distribution\n");
  lines.push("Based on Facebook USR/APP/VAR pools and Twitter/Meta traces:\n");
  lines.push("| Tier | Size | Items% | Size% |");
  lines.push("|------|------|--------|-------|");
  for (const t of VALUE_TIERS) {
    lines.push(`| ${t.name} | ${t.range} | ${t.items_pct}% | ~${t.size_pct}% |`);
  }
  lines.push("");

  lines.push("#### Operation Mix\n");
  lines.push("| Op | % | Source |");
  lines.push("|----|---|--------|");
  for (const o of OP_DIST) {
    lines.push(`| ${o.op} | ${o.pct}% | ${o.source} |`);
  }
  lines.push("");

  lines.push("#### Environment\n");
  lines.push("- OS: " + sys.osName + " (" + sys.arch + ")");
  lines.push("- CPU: " + sys.cpu);
  lines.push("- Cores: " + sys.cores);
  lines.push("- Memory: " + sys.mem + "GB");
  lines.push("- Rust: " + sys.rustVer + "\n");

  lines.push("#### Why Effective OPS?\n");
  lines.push("Raw OPS ignores hit rate — a cache with 99% hit rate at 1M ops/s outperforms one with 50% hit rate at 2M ops/s in real workloads.\n");
  lines.push("**Effective OPS** models real-world performance by penalizing cache misses with actual I/O latency.\n");
  lines.push("");
  lines.push("#### Why NVMe Latency?\n");
  lines.push("LRU caches typically sit in front of persistent storage (databases, KV stores). On cache miss, data must be fetched from disk.\n");
  lines.push(`Miss penalty: ${data.miss_latency_ns.toLocaleString()}ns — measured via ${data.miss_latency_method}\n`);
  lines.push("");
  lines.push("Formula: `effective_ops = 1 / (hit_time + miss_rate × miss_latency)`\n");
  lines.push("- hit_time = 1 / raw_ops\n");
  lines.push("- Higher hit rate → fewer disk reads → better effective throughput\n");

  lines.push("#### References\n");
  lines.push("- [cache_dataset](https://github.com/cacheMon/cache_dataset)");
  lines.push("- OSDI'20: Twitter cache analysis");
  lines.push("- FAST'20: Facebook RocksDB workloads");
  lines.push("- ATC'13: Scaling Memcache at Facebook");

  return lines.join("\n");
};

const genMdZh = (data, sys) => {
  const { config: cfg, stats } = data;
  const lines = [];

  lines.push("## LRU 缓存评测\n");
  lines.push("模拟真实数据分布，固定内存预算，对比命中率和有效吞吐。\n");

  const best = data.results[0];
  lines.push("### 结果\n");
  lines.push("| 库 | 命中率 | 有效吞吐 | 性能 | 内存 |");
  lines.push("|-----|--------|----------|------|------|");

  for (const r of data.results) {
    const perfPct = ((r.effective_ops / best.effective_ops) * 100).toFixed(0);
    lines.push(
      `| ${r.lib} | ${(r.hit_rate * 100).toFixed(2)}% | ${(r.effective_ops / 1e6).toFixed(2)}M/s | ${perfPct}% | ${r.memory_kb.toFixed(1)}KB |`
    );
  }
  lines.push("");

  lines.push("### 配置\n");
  lines.push(`内存: ${fmtSize(cfg.mem_budget)} · Zipf s=${cfg.zipf_s} · 读/写/删: ${cfg.read_ratio}/${cfg.write_ratio}/${cfg.delete_ratio}% · 未命中: ${cfg.real_miss_ratio}% · 操作: ${(cfg.ops_per_loop / 1e6).toFixed(0)}M×${cfg.loops}\n`);

  lines.push("### 大小分布\n");
  lines.push("| 范围 | 条目 | 容量 |");
  lines.push("|------|------|------|");
  for (const b of stats.size_distribution) {
    if (b.count > 0) {
      lines.push(`| ${b.label} | ${b.percent.toFixed(2)}% | ${b.size_percent.toFixed(2)}% |`);
    }
  }
  lines.push("");

  lines.push("---\n");
  lines.push("### 备注\n");
  lines.push("#### 数据分布\n");
  lines.push("基于 Facebook USR/APP/VAR 池和 Twitter/Meta 追踪数据：\n");
  const tierNamesZh = ["微小元数据", "小型结构体", "中型内容", "大型对象", "巨型数据"];
  lines.push("| 层级 | 大小 | 条目% | 容量% |");
  lines.push("|------|------|-------|-------|");
  for (let i = 0; i < VALUE_TIERS.length; i++) {
    const t = VALUE_TIERS[i];
    lines.push(`| ${tierNamesZh[i]} | ${t.range} | ${t.items_pct}% | ~${t.size_pct}% |`);
  }
  lines.push("");

  lines.push("#### 操作分布\n");
  const opNamesZh = ["读取", "写入", "删除"];
  lines.push("| 操作 | % | 来源 |");
  lines.push("|------|---|------|");
  for (let i = 0; i < OP_DIST.length; i++) {
    const o = OP_DIST[i];
    lines.push(`| ${opNamesZh[i]} | ${o.pct}% | ${o.source} |`);
  }
  lines.push("");

  lines.push("#### 环境\n");
  lines.push("- 系统: " + sys.osName + " (" + sys.arch + ")");
  lines.push("- CPU: " + sys.cpu);
  lines.push("- 核心数: " + sys.cores);
  lines.push("- 内存: " + sys.mem + "GB");
  lines.push("- Rust版本: " + sys.rustVer + "\n");

  lines.push("#### 为什么用有效吞吐？\n");
  lines.push("原始 OPS 忽略了命中率 — 一个 99% 命中率、1M ops/s 的缓存，实际性能远超 50% 命中率、2M ops/s 的缓存。\n");
  lines.push("**有效吞吐**通过对缓存未命中施加真实 I/O 延迟惩罚，模拟真实场景性能。\n");
  lines.push("");
  lines.push("#### 为什么用 NVMe 延迟？\n");
  lines.push("LRU 缓存通常位于持久化存储（数据库、KV 存储）前面。缓存未命中时，必须从磁盘读取数据。\n");
  lines.push(`未命中惩罚: ${data.miss_latency_ns.toLocaleString()}ns — 通过 ${data.miss_latency_method} 实测\n`);
  lines.push("");
  lines.push("公式: `有效吞吐 = 1 / (命中时间 + 未命中率 × 未命中延迟)`\n");
  lines.push("- 命中时间 = 1 / 原始吞吐\n");
  lines.push("- 命中率越高 → 磁盘读取越少 → 有效吞吐越高\n");

  lines.push("#### 参考\n");
  lines.push("- [cache_dataset](https://github.com/cacheMon/cache_dataset)");
  lines.push("- OSDI'20: Twitter 缓存分析");
  lines.push("- FAST'20: Facebook RocksDB 负载");
  lines.push("- ATC'13: Facebook Memcache 扩展");

  return lines.join("\n");
};

const main = () => {
  const data = JSON.parse(readFileSync(JSON_PATH, "utf8"));
  const sys = getSystemInfo();

  printConfig(data.config);
  printStats(data.stats);
  printSizeDistribution(data.stats.size_distribution);
  printConsoleTable(data.results);

  const enMd = genMdEn(data, sys);
  writeFileSync(EN_MD, enMd);
  console.log(`\nWritten: ${EN_MD}`);

  const zhMd = genMdZh(data, sys);
  writeFileSync(ZH_MD, zhMd);
  console.log(`Written: ${ZH_MD}`);
};

main();
