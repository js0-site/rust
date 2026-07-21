#!/usr/bin/env bun

import { readFileSync, writeFileSync, existsSync } from "fs";
import { join } from "path";
import { execSync } from "child_process";

const ROOT = import.meta.dirname;
const REGRESS_JSON = join(ROOT, "benches/regress.json");
const REGRESS_HTML = join(ROOT, "benches/regress.html");
const MAX_HISTORY = 128;

const BENCHMARKS = [
  { name: "encode", key: "file_storage/encode", label: "encode()" },
  { name: "decode", key: "file_storage/decode", label: "decode()" },
  { name: "roundtrip", key: "file_storage/roundtrip", label: "roundtrip()" },
];

function parseAllBenchmarks(output) {
  const results = {};
  const normalized = output.replace(/\r?\n\s*/g, " ");
  // Match: file_storage/encode time: [74.116 µs 74.346 µs 74.595 µs]
  const regex =
    /([a-z_]+\/[a-z0-9_]+)\s+time:\s+\[([\d.]+)\s+([^\s\]]+)\s+([\d.]+)\s+([^\s\]]+)\s+([\d.]+)\s+([^\s\]]+)\s*\]/gi;
  let match;
  while ((match = regex.exec(normalized)) !== null) {
    const key = match[1];
    let time = parseFloat(match[4]); // mean value
    const unit = match[5];
    // 转换为纳秒 / convert to nanoseconds
    if (unit === "ps") time /= 1000;
    else if (unit.includes("µ") || unit.includes("us") || unit === "µs") time *= 1000;
    else if (unit.includes("ms")) time *= 1000000;
    else if (unit === "s") time *= 1000000000;
    results[key] = time;
  }
  return results;
}

function formatTime(ns) {
  if (!ns) return "N/A";
  if (ns < 1000) return `${ns.toFixed(2)} ns`;
  if (ns < 1000000) return `${(ns / 1000).toFixed(2)} µs`;
  if (ns < 1000000000) return `${(ns / 1000000).toFixed(2)} ms`;
  return `${(ns / 1000000000).toFixed(2)} s`;
}

function formatDiff(curr, prev) {
  if (!prev || prev === 0 || !curr) return "";
  const diff = ((curr - prev) / prev) * 100;
  const sign = diff >= 0 ? "+" : "";
  const color = diff <= -1 ? "32" : diff >= 1 ? "31" : "33";
  return `\x1b[${color}m${sign}${diff.toFixed(1)}%\x1b[0m`;
}

// 计算每秒百万操作 (10k items per bench)
// Mops = 10000 / (time_ns / 1e9) / 1e6 = 10000 * 1e3 / time_ns
function formatMops(ns) {
  if (!ns) return "N/A";
  const mops = (10000 * 1000) / ns;
  return `${mops.toFixed(2)} M/s`;
}

function genHtml(entry, history) {
  const colors = ["#00d4ff", "#0f0", "#ff6b6b"];
  const latestMetrics = BENCHMARKS.map((b, i) => {
    const val = entry[b.name];
    const prev = history.length >= 2 ? history[history.length - 2][b.name] : null;
    const diff = prev && val ? (((val - prev) / prev) * 100).toFixed(1) : null;
    const diffClass = diff
      ? parseFloat(diff) <= -1
        ? "improved"
        : parseFloat(diff) >= 1
          ? "regressed"
          : "neutral"
      : "";
    return `
      <div class="metric">
        <div class="label">${b.label}</div>
        <div class="value">${formatTime(val)}</div>
        ${diff !== null ? `<div class="diff ${diffClass}">${parseFloat(diff) >= 0 ? "+" : ""}${diff}%</div>` : ""}
      </div>`;
  }).join("");

  const datasets = BENCHMARKS.map(
    (b, i) => `{
    label: "${b.label}",
    data: data.map(d => d.${b.name}),
    borderColor: "${colors[i]}",
    backgroundColor: "${colors[i]}22",
    fill: false,
    tension: 0.3
  }`,
  ).join(",\n        ");

  return `<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <title>u64_2 Performance Regression</title>
  <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
  <style>
    body { font-family: -apple-system, sans-serif; margin: 20px; background: #1a1a2e; color: #eee; }
    h1 { color: #00d4ff; margin-bottom: 5px; }
    .subtitle { color: #888; margin-bottom: 20px; }
    .latest { background: #16213e; padding: 15px; border-radius: 8px; margin: 20px 0; }
    .latest h3 { margin-top: 0; color: #00d4ff; }
    .metrics { display: flex; flex-wrap: wrap; gap: 15px; }
    .metric { background: #0f3460; padding: 12px 16px; border-radius: 6px; min-width: 140px; }
    .metric .label { font-size: 11px; color: #888; margin-bottom: 4px; }
    .metric .value { font-size: 18px; font-weight: bold; color: #fff; }
    .metric .diff { font-size: 12px; margin-top: 4px; }
    .metric .diff.improved { color: #0f0; }
    .metric .diff.regressed { color: #f66; }
    .metric .diff.neutral { color: #ff0; }
    .chart-container { width: 100%; max-width: 1200px; margin: 30px 0; }
    .note { color: #666; font-size: 12px; margin-top: 10px; }
  </style>
</head>
<body>
  <h1>u64_2 Performance Regression</h1>
  <div class="subtitle">Commit: ${entry.commit} (${entry.branch}) | ${entry.date}</div>
  <div class="latest">
    <h3>Latest Results (10k file entries)</h3>
    <div class="metrics">${latestMetrics}</div>
    <div class="note">Lower is better. Green = improved, Red = regressed vs previous.</div>
  </div>
  <div class="chart-container"><canvas id="chart"></canvas></div>
  <script>
    const data = ${JSON.stringify(history)};
    new Chart(document.getElementById("chart"), {
      type: "line",
      data: {
        labels: data.map(d => d.commit),
        datasets: [${datasets}]
      },
      options: {
        responsive: true,
        interaction: { mode: "index", intersect: false },
        plugins: {
          legend: { labels: { color: "#eee" }, position: "bottom" },
          title: { display: true, text: "Execution Time (ns) - Lower is Better", color: "#eee" }
        },
        scales: {
          x: { ticks: { color: "#888" }, grid: { color: "#333" } },
          y: { ticks: { color: "#888" }, grid: { color: "#333" }, type: "logarithmic" }
        }
      }
    });
  </script>
</body>
</html>`;
}

// ============ Main ============
console.log("🚀 Running u64_2 benchmarks...\n");

const commit = execSync("git rev-parse --short HEAD 2>/dev/null || echo unknown", {
  encoding: "utf8",
}).trim();
const branch = execSync("git branch --show-current 2>/dev/null || echo unknown", {
  encoding: "utf8",
}).trim();
const date = new Date().toISOString().replace("T", " ").slice(0, 19);

const entry = { date, commit, branch };

try {
  const output = execSync("cargo bench --bench u64_encode_decode 2>&1", {
    cwd: ROOT,
    encoding: "utf8",
    timeout: 300000,
  });
  const results = parseAllBenchmarks(output);
  for (const b of BENCHMARKS) {
    entry[b.name] = results[b.key] || null;
  }
} catch (e) {
  console.error("⚠ Benchmarks failed:", e.message);
  process.exit(1);
}

// 加载历史 / load history
let history = existsSync(REGRESS_JSON) ? JSON.parse(readFileSync(REGRESS_JSON, "utf8")) : [];
const prev = history.length > 0 ? history[history.length - 1] : null;

// 保存 / save
history.push(entry);
if (history.length > MAX_HISTORY) history = history.slice(-MAX_HISTORY);
writeFileSync(REGRESS_JSON, JSON.stringify(history, null, 2));
writeFileSync(REGRESS_HTML, genHtml(entry, history));

// 输出结果 / print results
console.log("=".repeat(50));
console.log(`📊 u64_2 Performance Report`);
console.log(`   Commit: ${commit} (${branch})`);
console.log(`   Date:   ${date}`);
console.log("=".repeat(50));
console.log("\n📈 Results (vs previous):\n");

const colW = 14;
console.log(
  `${"Function".padEnd(colW)} ${"Time".padStart(12)} ${"Mops".padStart(12)} ${"Change".padStart(12)}`,
);
console.log("-".repeat(colW + 38));

for (const b of BENCHMARKS) {
  const curr = entry[b.name];
  const prevVal = prev ? prev[b.name] : null;
  console.log(
    `${b.label.padEnd(colW)} ${formatTime(curr).padStart(12)} ${formatMops(curr).padStart(12)} ${formatDiff(curr, prevVal)}`,
  );
}

console.log(`\n📄 JSON: ${REGRESS_JSON}`);
console.log(`📄 HTML: ${REGRESS_HTML}`);
console.log("✅ Done.\n");
