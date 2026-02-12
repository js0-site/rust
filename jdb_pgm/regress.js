#!/usr/bin/env bun

import { readFileSync, writeFileSync, existsSync } from "fs";
import { join } from "path";
import { execSync } from "child_process";
import Table from "cli-table3";

const ROOT = import.meta.dirname;
const REGRESS_JSON = join(ROOT, "benches/regress.json");
const REGRESS_HTML = join(ROOT, "benches/regress.html");

const genHtml = (entry, history) => `<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <title>Pgm-Index Regression (PC)</title>
  <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
  <style>
    body { font-family: -apple-system, font-family: 'PingFang SC', sans-serif; margin: 20px; background: #1a1a2e; color: #eee; }
    h1 { color: #10b981; }
    .chart-container { width: 100%; max-width: 900px; margin: 20px 0; }
    .latest { background: #16213e; padding: 15px; border-radius: 8px; margin: 20px 0; }
    .latest h3 { margin-top: 0; color: #10b981; }
    .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(180px, 1fr)); gap: 15px; }
    .metric { margin-bottom: 5px; }
    .metric .value { font-size: 20px; font-weight: bold; color: #0f0; }
    .metric .label { font-size: 12px; color: #888; }
  </style>
</head>
<body>
  <h1>Pgm-Index PC 回归测试报告</h1>

  <div class="latest">
    <h3>最新提交: ${entry.commit} (${entry.branch}) - ${entry.date}</h3>
    <div class="grid">
      <div class="metric"><div class="value">${entry.comp_ratio.toFixed(2)}%</div><div class="label">压缩率</div></div>
      <div class="metric"><div class="value">${entry.build_mb_s.toFixed(2)} MB/s</div><div class="label">构建速度</div></div>
      <div class="metric"><div class="value">${entry.random_mb_s.toFixed(2)} MB/s</div><div class="label">随机访问</div></div>
      <div class="metric"><div class="value">${entry.seq_mb_s.toFixed(2)} MB/s</div><div class="label">顺序扫描</div></div>
      <div class="metric"><div class="value">${entry.latency_avg_ns.toFixed(1)} ns</div><div class="label">平均延迟</div></div>
      <div class="metric"><div class="value">${entry.latency_p99_ns.toFixed(0)} ns</div><div class="label">P99 延迟</div></div>
    </div>
  </div>

  <div class="chart-container"><canvas id="perfChart"></canvas></div>
  <div class="chart-container"><canvas id="latencyChart"></canvas></div>

  <script>
    const data = ${JSON.stringify(history)};
    const labels = data.map(d => d.commit);

    const chartOpts = {
      responsive: true,
      plugins: { legend: { labels: { color: '#eee' } } },
      scales: {
        x: { ticks: { color: '#888' }, grid: { color: '#333' } },
        y: { ticks: { color: '#888' }, grid: { color: '#333' } }
      }
    };

    new Chart(document.getElementById('perfChart'), {
      type: 'line',
      data: {
        labels,
        datasets: [
          {
            label: '随机访问 (MB/s)',
            data: data.map(d => d.random_mb_s),
            borderColor: '#10b981',
            backgroundColor: 'rgba(16,185,129,0.1)',
            fill: false,
            tension: 0.3
          },
          {
            label: '顺序扫描 (MB/s)',
            data: data.map(d => d.seq_mb_s),
            borderColor: '#3b82f6',
            backgroundColor: 'rgba(59,130,246,0.1)',
            fill: false,
            tension: 0.3
          },
          {
            label: '构建速度 (MB/s)',
            data: data.map(d => d.build_mb_s),
            borderColor: '#8b5cf6',
            backgroundColor: 'rgba(139,92,246,0.1)',
            fill: false,
            tension: 0.3
          }
        ]
      },
      options: { ...chartOpts, plugins: { ...chartOpts.plugins, title: { display: true, text: '吞吐性能 (MB/s)', color: '#eee' } } }
    });

    new Chart(document.getElementById('latencyChart'), {
      type: 'line',
      data: {
        labels,
        datasets: [
          {
            label: 'Avg 延迟 (ns)',
            data: data.map(d => d.latency_avg_ns),
            borderColor: '#f59e0b',
            backgroundColor: 'rgba(245,158,11,0.1)',
            fill: false,
            tension: 0.3
          },
          {
            label: 'P99 延迟 (ns)',
            data: data.map(d => d.latency_p99_ns),
            borderColor: '#ef4444',
            backgroundColor: 'rgba(239,68,68,0.1)',
            fill: false,
            tension: 0.3,
            borderDash: [5, 5]
          }
        ]
      },
      options: { ...chartOpts, plugins: { ...chartOpts.plugins, title: { display: true, text: '延迟 (ns)', color: '#eee' } } }
    });
  </script>
</body>
</html>`;

try {
  // Run example/pc with 1GB data
  // 运行 example/pc，数据量 1GB
  const output = execSync(`cargo run --release --example pc -- -n 131072000`, {
    cwd: ROOT,
    stdio: ["ignore", "pipe", "inherit"], // capture stdout
    maxBuffer: 10 * 1024 * 1024,
  }).toString();

  // Parse Output
  let metrics = {
    random_mb_s: 0,
    seq_mb_s: 0,
    build_mb_s: 0,
    comp_ratio: 0,
    latency_avg_ns: 0,
    latency_p99_ns: 0,
  };

  const lines = output.split("\n");
  let currentSection = "";

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i].trim();
    if (line.includes("随机 (MB/s):")) currentSection = "random";
    else if (line.includes("顺序 (MB/s):")) currentSection = "seq";
    else if (line.includes("构建 (MB/s):")) currentSection = "build";
    else if (line.includes("压缩率 (%):")) currentSection = "comp";
    else if (line.includes("延迟 Avg (ns):")) currentSection = "lat_avg";
    else if (line.includes("延迟 P99 (ns):")) currentSection = "lat_p99";
    else if (
      line.includes("(MB/s):") ||
      line.includes("(ns):") ||
      line.includes("大小") ||
      line.includes("逆向")
    ) {
      currentSection = ""; // Reset
    }

    if (line.startsWith("pc:")) {
      const val = parseFloat(line.split(":")[1].trim());
      if (currentSection === "random") metrics.random_mb_s = val;
      if (currentSection === "seq") metrics.seq_mb_s = val;
      if (currentSection === "build") metrics.build_mb_s = val;
      if (currentSection === "comp") metrics.comp_ratio = val;
      if (currentSection === "lat_avg") metrics.latency_avg_ns = val;
      if (currentSection === "lat_p99") metrics.latency_p99_ns = val;
    }
  }

  if (metrics.random_mb_s === 0) {
    console.error("错误: 无法从输出解析指标。");
    console.log("Output sample:", output.slice(0, 1000));
    process.exit(1);
  }

  const commit = execSync(
    "git rev-parse --short HEAD 2>/dev/null || echo unknown",
  )
    .toString()
    .trim();
  const branch = execSync(
    "git branch --show-current 2>/dev/null || echo unknown",
  )
    .toString()
    .trim();
  const date = new Date().toISOString().replace("T", "_").slice(0, 19);

  const entry = {
    date,
    commit,
    branch,
    data_size: 131072000 * 8, // 1GB
    ...metrics,
  };

  let history = existsSync(REGRESS_JSON)
    ? JSON.parse(readFileSync(REGRESS_JSON, "utf8"))
    : [];

  // Backward compatibility: filter out old format entries to avoid chart glitches if essential fields missing
  history = history.filter((h) => h.random_mb_s !== undefined);

  history.push(entry);
  if (history.length > 128) history = history.slice(-128);
  writeFileSync(REGRESS_JSON, JSON.stringify(history, null, 2));

  const html = genHtml(entry, history);
  writeFileSync(REGRESS_HTML, html);

  // Console Report with cli-table3

  console.log(`
数据量:     ${(entry.data_size / 1024 / 1024).toFixed(0)} MB
`);

  const prev = history.length >= 2 ? history[history.length - 2] : null;

  const table = new Table({
    head: ["指标", "当前值", "同比变化 (vs Prev)"],
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
    style: { pl: 0, pr: 2, head: ["green"] },
    colAligns: ["left", "right", "left"],
  });

  const addRow = (label, cur, unit, prevVal, smallerIsBetter = false) => {
    let diffStr = "N/A";
    if (prevVal !== undefined && prevVal !== 0) {
      const diff = ((cur - prevVal) / prevVal) * 100;
      const sign = diff >= 0 ? "+" : "";
      const color = smallerIsBetter
        ? diff < 0
          ? "\x1b[32m"
          : diff > 0
            ? "\x1b[31m"
            : "" // Lower green
        : diff > 0
          ? "\x1b[32m"
          : diff < 0
            ? "\x1b[31m"
            : ""; // Higher green
      const reset = "\x1b[0m";
      diffStr = `${color}${sign}${diff.toFixed(1)}%${reset}`;
    }
    table.push([label, `${cur.toFixed(2)} ${unit}`, diffStr]);
  };

  const p = prev || {};
  addRow("压缩率 (Comp Ratio)", entry.comp_ratio, "%", p.comp_ratio, true);
  addRow("构建速度 (Build)", entry.build_mb_s, "MB/s", p.build_mb_s, false);
  addRow("随机访问 (Random)", entry.random_mb_s, "MB/s", p.random_mb_s, false);
  addRow("顺序扫描 (Seq)", entry.seq_mb_s, "MB/s", p.seq_mb_s, false);
  addRow(
    "平均延迟 (Lat Avg)",
    entry.latency_avg_ns,
    "ns",
    p.latency_avg_ns,
    true,
  );
  addRow(
    "P99 延迟 (Lat P99)",
    entry.latency_p99_ns,
    "ns",
    p.latency_p99_ns,
    true,
  );

  console.log(table.toString());

  console.log(`\nHTML 报告: ${REGRESS_HTML}`);
  // await $`open ${REGRESS_HTML}`;
} catch (e) {
  console.error("Benchmark failed:", e.message);
  process.exit(1);
}
