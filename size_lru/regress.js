#!/usr/bin/env bun

import { readFileSync, writeFileSync, existsSync } from 'fs';
import { join } from 'path';
import { execSync } from 'child_process';
import { $ } from 'bun';

const ROOT = import.meta.dirname;
const BENCH_JSON = join(ROOT, 'bench.json');
const REGRESS_JSON = join(ROOT, 'benches/regress.json');
const REGRESS_HTML = join(ROOT, 'benches/regress.html');

const genHtml = (entry, history) => `<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <title>size_lru Regression</title>
  <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
  <style>
    body { font-family: -apple-system, sans-serif; margin: 20px; background: #1a1a2e; color: ee; }
    h1 { color: #00d4ff; }
    .chart-container { width: 100%; max-width: 900px; margin: 20px 0; }
    .latest { background: #16213e; padding: 15px; border-radius: 8px; margin: 20px 0; }
    .latest h3 { margin-top: 0; color: #00d4ff; }
    .metric { display: inline-block; margin-right: 30px; }
    .metric .value { font-size: 24px; font-weight: bold; color: #0f0; }
    .metric .label { font-size: 12px; color: #888; }
  </style>
</head>
<body>
  <h1>size_lru Regression Test</h1>
  
  <div class="latest">
    <h3>Latest: ${entry.commit} (${entry.branch}) - ${entry.date}</h3>
    <div class="metric">
      <div class="value">${entry.eff_ops.toFixed(2)} M/s</div>
      <div class="label">Effective OPS</div>
    </div>
    <div class="metric">
      <div class="value">${entry.hit_rate.toFixed(1)}%</div>
      <div class="label">Hit Rate</div>
    </div>
    <div class="metric">
      <div class="value">${(entry.memory_kb/1024).toFixed(1)} MB</div>
      <div class="label">Memory</div>
    </div>
    <div class="metric">
      <div class="value">${entry.raw_ops.toFixed(2)} M/s</div>
      <div class="label">Raw OPS</div>
    </div>
  </div>

  <div class="chart-container"><canvas id="opsChart"></canvas></div>
  <div class="chart-container"><canvas id="hitChart"></canvas></div>
  <div class="chart-container"><canvas id="memChart"></canvas></div>

  <script>
    const data = ${JSON.stringify(history)};
    const labels = data.map(d => d.commit);
    
    const chartOpts = {
      responsive: true,
      plugins: { legend: { labels: { color: 'ee' } } },
      scales: {
        x: { ticks: { color: '#888' }, grid: { color: '#333' } },
        y: { ticks: { color: '#888' }, grid: { color: '#333' } }
      }
    };

    new Chart(document.getElementById('opsChart'), {
      type: 'line',
      data: {
        labels,
        datasets: [{
          label: 'Effective OPS (M/s)',
          data: data.map(d => d.eff_ops),
          borderColor: '#00d4ff',
          backgroundColor: 'rgba(0,212,255,0.1)',
          fill: true,
          tension: 0.3
        }]
      },
      options: { ...chartOpts, plugins: { ...chartOpts.plugins, title: { display: true, text: 'Effective OPS', color: 'ee' } } }
    });

    new Chart(document.getElementById('hitChart'), {
      type: 'line',
      data: {
        labels,
        datasets: [{
          label: 'Hit Rate (%)',
          data: data.map(d => d.hit_rate),
          borderColor: '#0f0',
          backgroundColor: 'rgba(0,255,0,0.1)',
          fill: true,
          tension: 0.3
        }]
      },
      options: { ...chartOpts, plugins: { ...chartOpts.plugins, title: { display: true, text: 'Hit Rate', color: 'ee' } } }
    });

    new Chart(document.getElementById('memChart'), {
      type: 'line',
      data: {
        labels,
        datasets: [{
          label: 'Memory (KB)',
          data: data.map(d => d.memory_kb),
          borderColor: '#ff6b6b',
          backgroundColor: 'rgba(255,107,107,0.1)',
          fill: true,
          tension: 0.3
        }]
      },
      options: { ...chartOpts, plugins: { ...chartOpts.plugins, title: { display: true, text: 'Memory Usage', color: 'ee' } } }
    });
  </script>
</body>
</html>`;

console.log('Running size_lru benchmark...');

execSync('cargo bench --bench comparison --features bench-size-lru -- --nocapture', {
  cwd: ROOT,
  stdio: ['inherit', 'ignore', 'ignore']
});

if (!existsSync(BENCH_JSON)) {
  console.error(`Error: ${BENCH_JSON} not found`);
  process.exit(1);
}

const bench = JSON.parse(readFileSync(BENCH_JSON, 'utf8'));
const r = bench.results.find(x => x.lib === 'size_lru');

if (!r) {
  console.error('Error: size_lru result not found');
  process.exit(1);
}

const commit = execSync('git rev-parse --short HEAD 2>/dev/null || echo unknown').toString().trim();
const branch = execSync('git branch --show-current 2>/dev/null || echo unknown').toString().trim();
const date = new Date().toISOString().replace('T', '_').slice(0, 19);

const entry = {
  date,
  commit,
  branch,
  hit_rate: r.hit_rate * 100,
  eff_ops: r.effective_ops / 1e6,
  raw_ops: r.ops_per_second / 1e6,
  memory_kb: r.memory_kb,
};

let history = existsSync(REGRESS_JSON) ? JSON.parse(readFileSync(REGRESS_JSON, 'utf8')) : [];
history.push(entry);
if (history.length > 128) history = history.slice(-128);
writeFileSync(REGRESS_JSON, JSON.stringify(history, null, 2));

const html = genHtml(entry, history);
writeFileSync(REGRESS_HTML, html);

console.log(`

=== size_lru Regression Result ===
Date:      ${entry.date}
Commit:    ${entry.commit} (${entry.branch})
Hit Rate:  ${entry.hit_rate.toFixed(2)}%
Eff OPS:   ${entry.eff_ops.toFixed(2)} M/s
Raw OPS:   ${entry.raw_ops.toFixed(2)} M/s
Memory:    ${entry.memory_kb.toFixed(1)} KB`);

if (history.length >= 2) {
  const prev = history[history.length - 2];
  const diff = ((entry.eff_ops - prev.eff_ops) / prev.eff_ops * 100).toFixed(1);
  const sign = diff >= 0 ? '+' : '';
  console.log(`vs prev:   ${sign}${diff}% eff_ops`);
}

console.log(`

HTML: ${REGRESS_HTML}`);

await $`open ${REGRESS_HTML}`;

console.log('Done.');