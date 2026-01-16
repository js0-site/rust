#!/usr/bin/env bun

import { readFileSync, writeFileSync, existsSync } from "fs";
import { join } from "path";
import { execSync } from "child_process";
import { $ } from "bun";
import { fmtTime, fmtThroughput, formatDataSize } from "./js/common.js";

const ROOT = import.meta.dirname;
const BENCH_JSON = join(ROOT, "bench.json");
const REGRESS_JSON = join(ROOT, "benches/regress.json");
const REGRESS_HTML = join(ROOT, "benches/regress.html");

const genHtml = (entry, history) => `<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <title>Pgm-Index Regression</title>
  <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
  <style>
    body { font-family: -apple-system, sans-serif; margin: 20px; background: #1a1a2e; color: #eee; }
    h1 { color: #10b981; }
    .chart-container { width: 100%; max-width: 900px; margin: 20px 0; }
    .latest { background: #16213e; padding: 15px; border-radius: 8px; margin: 20px 0; }
    .latest h3 { margin-top: 0; color: #10b981; }
    .metric { display: inline-block; margin-right: 30px; }
    .metric .value { font-size: 24px; font-weight: bold; color: #0f0; }
    .metric .label { font-size: 12px; color: #888; }
  </style>
</head>
<body>
  <h1>Pgm-Index Regression Test</h1>

  <div class="latest">
    <h3>Latest: ${entry.commit} (${entry.branch}) - ${entry.date}</h3>
    <div class="metric">
      <div class="value">${entry.throughput.toFixed(2)} M/s</div>
      <div class="label">Throughput</div>
    </div>
    <div class="metric">
      <div class="value">${entry.mean_ns.toFixed(1)} ns</div>
      <div class="label">Mean Time</div>
    </div>
    <div class="metric">
      <div class="value">${entry.std_dev_ns.toFixed(1)} ns</div>
      <div class="label">Std Dev</div>
    </div>
    <div class="metric">
      <div class="value">${entry.data_size.toLocaleString()}</div>
      <div class="label">Data Size</div>
    </div>
  </div>

  <div class="chart-container"><canvas id="throughputChart"></canvas></div>
  <div class="chart-container"><canvas id="timeChart"></canvas></div>
  <div class="chart-container"><canvas id="stdDevChart"></canvas></div>

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

    new Chart(document.getElementById('throughputChart'), {
      type: 'line',
      data: {
        labels,
        datasets: [{
          label: 'Throughput (M/s)',
          data: data.map(d => d.throughput),
          borderColor: '#10b981',
          backgroundColor: 'rgba(16,185,129,0.1)',
          fill: true,
          tension: 0.3
        }]
      },
      options: { ...chartOpts, plugins: { ...chartOpts.plugins, title: { display: true, text: 'Throughput', color: '#eee' } } }
    });

    new Chart(document.getElementById('timeChart'), {
      type: 'line',
      data: {
        labels,
        datasets: [{
          label: 'Mean Time (ns)',
          data: data.map(d => d.mean_ns),
          borderColor: '#3b82f6',
          backgroundColor: 'rgba(59,130,246,0.1)',
          fill: true,
          tension: 0.3
        }]
      },
      options: { ...chartOpts, plugins: { ...chartOpts.plugins, title: { display: true, text: 'Mean Query Time', color: '#eee' } } }
    });

    new Chart(document.getElementById('stdDevChart'), {
      type: 'line',
      data: {
        labels,
        datasets: [{
          label: 'Std Dev (ns)',
          data: data.map(d => d.std_dev_ns),
          borderColor: '#f97316',
          backgroundColor: 'rgba(249,115,22,0.1)',
          fill: true,
          tension: 0.3
        }]
      },
      options: { ...chartOpts, plugins: { ...chartOpts.plugins, title: { display: true, text: 'Standard Deviation', color: '#eee' } } }
    });
  </script>
</body>
</html>`;

console.log("Running jdb_pgm regression benchmark...");

// Run benchmark (jdb_pgm only, no pk feature)
// 运行评测（仅 jdb_pgm，不启用 pk feature）
execSync(`cargo bench --bench main`, {
  cwd: ROOT,
  stdio: ["inherit", "inherit", "inherit"],
});

if (!existsSync(BENCH_JSON)) {
  console.error(`Error: ${BENCH_JSON} not found`);
  process.exit(1);
}

const bench = JSON.parse(readFileSync(BENCH_JSON, "utf8"));

// Use the largest data size with epsilon=64
// 使用最大数据量 epsilon=64
const targetDataSize = Math.max(...bench.config.data_sizes);
const r = bench.results.find(
  (x) =>
    x.algorithm === "jdb_pgm" &&
    x.epsilon === 64 &&
    x.data_size === targetDataSize,
);

if (!r) {
  console.error(
    `Error: jdb_pgm result not found (epsilon=64, data_size=${targetDataSize})`,
  );
  process.exit(1);
}

const commit = execSync(
  "git rev-parse --short HEAD 2>/dev/null || echo unknown",
)
  .toString()
  .trim();
const branch = execSync("git branch --show-current 2>/dev/null || echo unknown")
  .toString()
  .trim();
const date = new Date().toISOString().replace("T", "_").slice(0, 19);

const entry = {
  date,
  commit,
  branch,
  data_size: r.data_size,
  mean_ns: r.mean_ns,
  std_dev_ns: r.std_dev_ns,
  throughput: r.throughput / 1e6,
};

let history = existsSync(REGRESS_JSON)
  ? JSON.parse(readFileSync(REGRESS_JSON, "utf8"))
  : [];
history.push(entry);
if (history.length > 128) history = history.slice(-128);
writeFileSync(REGRESS_JSON, JSON.stringify(history, null, 2));

const html = genHtml(entry, history);
writeFileSync(REGRESS_HTML, html);

console.log(`

=== jdb_pgm Regression Result ===
Date:      ${entry.date}
Commit:    ${entry.commit} (${entry.branch})
Data Size: ${formatDataSize(entry.data_size)}
Mean Time: ${fmtTime(entry.mean_ns)}
Std Dev:   ${fmtTime(entry.std_dev_ns)}
Throughput: ${fmtThroughput(entry.throughput * 1e6)}`);

if (history.length >= 2) {
  const prev = history[history.length - 2];
  const diff = (
    ((entry.throughput - prev.throughput) / prev.throughput) *
    100
  ).toFixed(1);
  const sign = diff >= 0 ? "+" : "";
  console.log(`vs prev:   ${sign}${diff}% throughput`);
}

console.log(`

HTML: ${REGRESS_HTML}`);

await $`open ${REGRESS_HTML}`;

console.log("Done.");
