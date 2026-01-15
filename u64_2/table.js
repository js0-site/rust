#!/usr/bin/env bun

import { readFileSync, writeFileSync, existsSync, mkdirSync } from "fs";
import { join } from "path";
import { execSync } from "child_process";
import os from "os";
import Table from "cli-table3";

const ROOT = import.meta.dirname;
const JSON_PATH = join(ROOT, "bench.json");
const EN_MD = join(ROOT, "readme/en.bench.md");
const ZH_MD = join(ROOT, "readme/zh.bench.md");

const getSystemInfo = () => {
  const cpus = os.cpus();
  const cpu = cpus[0]?.model || "Unknown";
  const cores = cpus.length;
  const mem = (os.totalmem() / 1024 / 1024 / 1024).toFixed(1);
  const platform = os.platform();
  const arch = os.arch();

  let rustVer = "Unknown";
  try {
    rustVer = execSync("rustc --version", { encoding: "utf8" }).trim();
  } catch {}

  let osName = `${platform}`;
  if (platform === "darwin") {
    try {
      const ver = execSync("sw_vers -productVersion", { encoding: "utf8" }).trim();
      osName = `macOS ${ver}`;
    } catch {}
  }

  return { cpu, cores, mem, osName, arch, rustVer };
};

const printConsoleTable = (results, dataCount) => {
  const table = new Table({
    head: ["Library", "Encode (M/s)", "Decode (M/s)"],
    style: { head: ["cyan"] },
  });

  for (const r of results) {
    const encThroughput = ((dataCount / r.encode_ns) * 1e9 / 1e6).toFixed(1);
    const decThroughput = ((dataCount / r.decode_ns) * 1e9 / 1e6).toFixed(1);
    table.push([r.lib, encThroughput, decThroughput]);
  }

  console.log(`\nu64_2 vs vb Benchmark (${dataCount.toLocaleString()} integers):`);
  console.log(table.toString());
};

const genMdEn = (data, sys) => {
  const { results, data_count } = data;
  const lines = [];

  lines.push("## u64_2 vs vb Benchmark\n");
  lines.push(`Comparing u64_2 (pair encoding) with vb (varint) using ${data_count.toLocaleString()} integers (mixed distribution: 60% small, 30% medium, 10% large).\n`);

  lines.push("### Results\n");
  lines.push("| Library | Encode (M/s) | Decode (M/s) |");
  lines.push("|---------|--------------|--------------|");

  for (const r of results) {
    const encThroughput = ((data_count / r.encode_ns) * 1e9 / 1e6).toFixed(1);
    const decThroughput = ((data_count / r.decode_ns) * 1e9 / 1e6).toFixed(1);
    lines.push(`| ${r.lib} | ${encThroughput} | ${decThroughput} |`);
  }
  lines.push("");

  lines.push("### Environment\n");
  lines.push(`${sys.osName} (${sys.arch}) · ${sys.cpu} · ${sys.cores} cores · ${sys.mem}GB · ${sys.rustVer}\n`);

  return lines.join("\n");
};

const genMdZh = (data, sys) => {
  const { results, data_count } = data;
  const lines = [];

  lines.push("## u64_2 vs vb 性能评测\n");
  lines.push(`对比 u64_2（双值编码）与 vb（变长编码），测试数据：${data_count.toLocaleString()} 个整数（混合分布：60% 小值，30% 中值，10% 大值）。\n`);

  lines.push("### 结果\n");
  lines.push("| 库 | 编码 (百万/秒) | 解码 (百万/秒) |");
  lines.push("|----|----------------|----------------|");

  for (const r of results) {
    const encThroughput = ((data_count / r.encode_ns) * 1e9 / 1e6).toFixed(1);
    const decThroughput = ((data_count / r.decode_ns) * 1e9 / 1e6).toFixed(1);
    lines.push(`| ${r.lib} | ${encThroughput} | ${decThroughput} |`);
  }
  lines.push("");

  lines.push("### 环境\n");
  lines.push(`${sys.osName} (${sys.arch}) · ${sys.cpu} · ${sys.cores} 核 · ${sys.mem}GB · ${sys.rustVer}\n`);

  return lines.join("\n");
};

const main = () => {
  const data = JSON.parse(readFileSync(JSON_PATH, "utf8"));
  const sys = getSystemInfo();

  printConsoleTable(data.results, data.data_count);

  if (!existsSync("readme")) mkdirSync("readme");

  const enMd = genMdEn(data, sys);
  writeFileSync(EN_MD, enMd);
  console.log(`\nWritten: ${EN_MD}`);

  const zhMd = genMdZh(data, sys);
  writeFileSync(ZH_MD, zhMd);
  console.log(`Written: ${ZH_MD}`);
};

main();
