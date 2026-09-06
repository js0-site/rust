#!/usr/bin/env bun

import { readFileSync, writeFileSync, existsSync } from "fs";
import { join } from "path";
import { execSync } from "child_process";
import ERR from "@3-/log/ERR.js";

const ROOT = import.meta.dirname,
  BENCH_JSON = join(ROOT, "bench.json"),
  REGRESS_JSON = join(ROOT, "benches/regress.json"),
  REGRESS_HTML = join(ROOT, "benches/regress.html");

const genHtml = (entry, history) => {
  return '<!DOCTYPE html>\n' +
    '<html>\n' +
    '<head>\n' +
    '  <meta charset="utf-8">\n' +
    '  <title>size_lru Regression</title>\n' +
    '  <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>\n' +
    '  <style>\n' +
    '    body { font-family: -apple-system, sans-serif; margin: 20px; background: #1a1a2e; color: #eee; }\n' +
    '    h1 { color: #00d4ff; }\n' +
    '    .chart-container { width: 100%; max-width: 900px; margin: 20px 0; }\n' +
    '    .latest { background: #16213e; padding: 15px; border-radius: 8px; margin: 20px 0; }\n' +
    '    .latest h3 { margin-top: 0; color: #00d4ff; }\n' +
    '    .metric { display: inline-block; margin-right: 30px; }\n' +
    '    .metric .value { font-size: 24px; font-weight: bold; color: #0f0; }\n' +
    '    .metric .label { font-size: 12px; color: #888; }\n' +
    '  </style>\n' +
    '</head>\n' +
    '<body>\n' +
    '  <h1>size_lru Regression Test</h1>\n' +
    '  \n' +
    '  <div class="latest">\n' +
    '    <h3>Latest: ' + entry.commit + ' (' + entry.branch + ') - ' + entry.date + '</h3>\n' +
    '    <div class="metric">\n' +
    '      <div class="value">' + entry.eff_ops.toFixed(2) + ' M/s</div>\n' +
    '      <div class="label">Effective OPS</div>\n' +
    '    </div>\n' +
    '    <div class="metric">\n' +
    '      <div class="value">' + entry.hit_rate.toFixed(1) + '%</div>\n' +
    '      <div class="label">Hit Rate</div>\n' +
    '    </div>\n' +
    '    <div class="metric">\n' +
    '      <div class="value">' + entry.memory_mb.toFixed(3) + ' MB</div>\n' +
    '      <div class="label">Memory</div>\n' +
    '    </div>\n' +
    '    <div class="metric">\n' +
    '      <div class="value">' + entry.raw_ops.toFixed(2) + ' M/s</div>\n' +
    '      <div class="label">Raw OPS</div>\n' +
    '    </div>\n' +
    '  </div>\n' +
    '\n' +
    '  <div class="chart-container"><canvas id="opsChart"></canvas></div>\n' +
    '  <div class="chart-container"><canvas id="hitChart"></canvas></div>\n' +
    '  <div class="chart-container"><canvas id="memChart"></canvas></div>\n' +
    '\n' +
    '  <script>\n' +
    '    const data = ' + JSON.stringify(history) + ';\n' +
    '    const labels = data.map(d => d.commit);\n' +
    '    \n' +
    '    const chartOpts = {\n' +
    '      responsive: true,\n' +
    '      plugins: { legend: { labels: { color: \'#eee\' } } },\n' +
    '      scales: {\n' +
    '        x: { ticks: { color: \'#888\' }, grid: { color: \'#333\' } },\n' +
    '        y: { ticks: { color: \'#888\' }, grid: { color: \'#333\' } }\n' +
    '      }\n' +
    '    };\n' +
    '\n' +
    '    new Chart(document.getElementById(\'opsChart\'), {\n' +
    '      type: \'line\',\n' +
    '      data: {\n' +
    '        labels,\n' +
    '        datasets: [{\n' +
    '          label: \'Effective OPS (M/s)\',\n' +
    '          data: data.map(d => d.eff_ops),\n' +
    '          borderColor: \'#00d4ff\',\n' +
    '          backgroundColor: \'rgba(0,212,255,0.1)\',\n' +
    '          fill: true,\n' +
    '          tension: 0.3\n' +
    '        }]\n' +
    '      },\n' +
    '      options: { ...chartOpts, plugins: { ...chartOpts.plugins, title: { display: true, text: \'Effective OPS\', color: \'#eee\' } } }\n' +
    '    });\n' +
    '\n' +
    '    new Chart(document.getElementById(\'hitChart\'), {\n' +
    '      type: \'line\',\n' +
    '      data: {\n' +
    '        labels,\n' +
    '        datasets: [{\n' +
    '          label: \'Hit Rate (%)\',\n' +
    '          data: data.map(d => d.hit_rate),\n' +
    '          borderColor: \'#0f0\',\n' +
    '          backgroundColor: \'rgba(0,255,0,0.1)\',\n' +
    '          fill: true,\n' +
    '          tension: 0.3\n' +
    '        }]\n' +
    '      },\n' +
    '      options: { ...chartOpts, plugins: { ...chartOpts.plugins, title: { display: true, text: \'Hit Rate\', color: \'#eee\' } } }\n' +
    '    });\n' +
    '\n' +
    '    new Chart(document.getElementById(\'memChart\'), {\n' +
    '      type: \'line\',\n' +
    '      data: {\n' +
    '        labels,\n' +
    '        datasets: [{\n' +
    '          label: \'Memory (MB)\',\n' +
    '          data: data.map(d => d.memory_mb),\n' +
    '          borderColor: \'#ff6b6b\',\n' +
    '          backgroundColor: \'rgba(255,107,107,0.1)\',\n' +
    '          fill: true,\n' +
    '          tension: 0.3\n' +
    '        }]\n' +
    '      },\n' +
    '      options: { ...chartOpts, plugins: { ...chartOpts.plugins, title: { display: true, text: \'Memory Usage\', color: \'#eee\' } } }\n' +
    '    });\n' +
    '  </script>\n' +
    '</body>\n' +
    '</html>';
};

console.log("Running size_lru benchmark...");

execSync("cargo bench --bench bench --features bench-size-lru -- --nocapture", {
  cwd: ROOT,
  stdio: ["inherit", "ignore", "ignore"],
});

if (!existsSync(BENCH_JSON)) {
  ERR("Error: " + BENCH_JSON + " not found");
  process.exit(1);
}

const bench = JSON.parse(readFileSync(BENCH_JSON, "utf8")),
  r = bench.results.find((x) => x.lib === "size_lru");

if (!r) {
  ERR("Error: size_lru result not found");
  process.exit(1);
}

const commit = execSync("git rev-parse --short HEAD 2>/dev/null || echo unknown").toString().trim(),
  branch = execSync("git branch --show-current 2>/dev/null || echo unknown").toString().trim(),
  date = new Date().toISOString().replace("T", "_").slice(0, 19),
  entry = {
    date,
    commit,
    branch,
    hit_rate: r.hit_rate * 100,
    eff_ops: r.effective_ops / 1e6,
    raw_ops: r.ops_per_second / 1e6,
    memory_mb: r.memory_mb,
  };

const history = existsSync(REGRESS_JSON) ? JSON.parse(readFileSync(REGRESS_JSON, "utf8")) : [];
history.push(entry);
if (history.length > 128) {
  history.splice(0, history.length - 128);
}
writeFileSync(REGRESS_JSON, JSON.stringify(history, null, 2));

const html = genHtml(entry, history);
writeFileSync(REGRESS_HTML, html);

console.log(
  "\n\n=== size_lru Regression Result ===\n" +
  "Date:      " + entry.date + "\n" +
  "Commit:    " + entry.commit + " (" + entry.branch + ")\n" +
  "Hit Rate:  " + entry.hit_rate.toFixed(2) + "%\n" +
  "Eff OPS:   " + entry.eff_ops.toFixed(2) + " M/s\n" +
  "Raw OPS:   " + entry.raw_ops.toFixed(2) + " M/s\n" +
  "Memory:    " + entry.memory_mb.toFixed(3) + " MB"
);

if (history.length >= 2) {
  const prev = history[history.length - 2],
    diff = (((entry.eff_ops - prev.eff_ops) / prev.eff_ops) * 100).toFixed(1),
    sign = diff >= 0 ? "+" : "";
  console.log("vs prev:   " + sign + diff + "% eff_ops");
}

console.log("\n\nHTML: " + REGRESS_HTML);

execSync("open " + REGRESS_HTML);

console.log("Done.");
