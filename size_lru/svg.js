#!/usr/bin/env bun

import { readFileSync, writeFileSync } from "fs";
import { optimize } from "svgo";

const data = JSON.parse(readFileSync("bench.json", "utf-8"));

const LIB_COLORS = {
  size_lru: "#f97316",
  moka: "#06b6d4",
  "mini-moka": "#8b5cf6",
  clru: "#10b981",
  lru: "#3b82f6",
  hashlink: "#ec4899",
  schnellru: "#64748b",
};
const DEFAULT_COLOR = "#a1a1aa";

// 主标题字体大小
const MAIN_TITLE_SIZE = 46;

const getColor = (lib) => LIB_COLORS[lib] || DEFAULT_COLOR;

const bar2d = (x, y, w, h, c) =>
  `<rect x="${x}" y="${y}" width="${w}" height="${h}" fill="${c}"/>`;

const barChart = (results, m, yOffset, getValue, formatLabel) => {
  const cH = m.chartH - m.t - m.b;
  const libs = results.map((r) => r.lib);

  let maxVal = 0;
  for (const r of results) {
    maxVal = Math.max(maxVal, getValue(r));
  }

  const bW = Math.min(60, (m.cW - 20) / libs.length - 10);
  const gap = (m.cW - libs.length * bW) / (libs.length + 1);

  let svg = "";

  for (let i = 0; i <= 4; i++) {
    const y = yOffset + m.t + cH - (cH * i) / 4;
    svg += `<line x1="${m.l}" y1="${y}" x2="${m.l + m.cW}" y2="${y}" stroke="0e0e0"/>`;
    const label = i === 4 ? formatLabel(maxVal) : formatLabel((maxVal * i) / 4);
    svg += `<text x="${m.l - 10}" y="${y + 6}" text-anchor="end" font-size="16" fill="#666">${label}</text>`;
  }
  svg += `<line x1="${m.l}" y1="${yOffset + m.t + cH}" x2="${m.l + m.cW}" y2="${yOffset + m.t + cH}" stroke="#888"/>`;

  results.forEach((r, i) => {
    const x = m.l + gap + i * (bW + gap);
    const v = getValue(r);
    const h = (v / maxVal) * cH;
    const y = yOffset + m.t + cH - h;
    svg += bar2d(x, y, bW, h, getColor(r.lib));
    svg += `<text x="${x + bW / 2}" y="${y - 8}" text-anchor="middle" font-size="12" fill="#333">${formatLabel(v)}</text>`;
    const labelY = yOffset + m.chartH - m.b + 20;
    svg += `<text x="${x + bW / 2}" y="${labelY}" text-anchor="end" font-size="14" fill="#333" transform="rotate(-45,${x + bW / 2},${labelY})">${r.lib}</text>`;
  });

  return svg;
};

const combine = (data, lang) => {
  const { results } = data;
  const libs = results.map((r) => r.lib);

  const m = { t: 70, r: 40, b: 100, l: 90, cW: 570, chartH: 320 };
  const W = m.l + m.cW + m.r;
  const chartGap = 40;
  const titleH = 90;

  const mainTitle = lang === "en" ? "Rust LRU Benchmark" : "Rust LRU 评测";
  const subTitle =
    lang === "en"
      ? "Real-world Data Distribution, Fixed Memory Budget"
      : "模拟真实世界数据分布，固定内存大小";

  const titles =
    lang === "en"
      ? ["Equivalent OPS (M/s)", "OPS (M/s)", "Hit Rate (%)", "Memory (KB)"]
      : ["等价 OPS (百万/秒)", "OPS (百万/秒)", "命中率 (%)", "内存 (KB)"];

  const sortedByEffectiveOps = [...results].sort(
    (a, b) => b.effective_ops - a.effective_ops,
  );
  const sortedByOps = [...results].sort(
    (a, b) => b.ops_per_second - a.ops_per_second,
  );
  const sortedByHitRate = [...results].sort((a, b) => b.hit_rate - a.hit_rate);
  const sortedByMemory = [...results].sort((a, b) => a.memory_kb - b.memory_kb);

  const charts = [
    {
      data: sortedByEffectiveOps,
      getValue: (r) => r.effective_ops / 1e6,
      format: (v) => v.toFixed(2),
    },
    {
      data: sortedByOps,
      getValue: (r) => r.ops_per_second / 1e6,
      format: (v) => v.toFixed(1),
    },
    {
      data: sortedByHitRate,
      getValue: (r) => r.hit_rate * 100,
      format: (v) => v.toFixed(1),
    },
    {
      data: sortedByMemory,
      getValue: (r) => r.memory_kb,
      format: (v) => v.toFixed(0),
    },
  ];

  const missLatencyUs = (data.miss_latency_ns / 1000).toFixed(1);
  const noteLine1 =
    lang === "en"
      ? `Equivalent OPS = 1 / (avg_op_time + miss_rate × miss_latency)`
      : `等价 OPS = 1 / (平均操作时间 + 未命中率 × 未命中延迟)`;
  const noteLine2 =
    lang === "en"
      ? `miss_latency = ${missLatencyUs}µs`
      : `未命中延迟 = ${missLatencyUs}µs`;
  const noteLine3 = data.miss_latency_method;
  const noteGap = 56;

  let svg = `<text x="${W / 2}" y="80" text-anchor="middle" font-size="${MAIN_TITLE_SIZE}" font-weight="bold" fill="#222">${mainTitle}</text>`;
  svg += `<text x="${W / 2}" y="${80 + MAIN_TITLE_SIZE * 0.8}" text-anchor="middle" font-size="22" fill="#666">${subTitle}</text>`;
  let yOffset = titleH + MAIN_TITLE_SIZE;

  charts.forEach((chart, idx) => {
    svg += `<text x="${W / 2}" y="${yOffset + 40}" text-anchor="middle" font-size="24" font-weight="bold" fill="#333">${titles[idx]}</text>`;
    svg += barChart(chart.data, m, yOffset, chart.getValue, chart.format);
    yOffset += m.chartH;
    if (idx === 0) {
      svg += `<text x="${W / 2}" y="${yOffset + 16}" text-anchor="middle" font-size="14" fill="#888">${noteLine1}</text>`;
      svg += `<text x="${W / 2}" y="${yOffset + 36}" text-anchor="middle" font-size="14" fill="#888">${noteLine2}</text>`;
      svg += `<text x="${W / 2}" y="${yOffset + 56}" text-anchor="middle" font-size="14" fill="#888">${noteLine3}</text>`;
      yOffset += noteGap;
    }
    yOffset += chartGap;
  });

  const legendY = yOffset;
  const legendCols = 4;
  const colW = m.cW / legendCols;
  let legend = "";
  libs.forEach((lib, i) => {
    const col = i % legendCols;
    const row = Math.floor(i / legendCols);
    const x = m.l + col * colW;
    const y = legendY + row * 32;
    legend += `<rect x="${x}" y="${y}" width="20" height="20" fill="${getColor(lib)}"/>`;
    legend += `<text x="${x + 28}" y="${y + 18}" font-size="18" fill="#333">${lib}</text>`;
  });

  const legendRows = Math.ceil(libs.length / legendCols);
  const H = yOffset + legendRows * 32 + 20;

  return `<svg xmlns="http://www.w3.org/2000/svg" width="${W}" height="${H}" viewBox="0 0 ${W} ${H}">
<rect width="${W}" height="${H}" fill="#fff"/>
${svg}
<g>${legend}</g>
</svg>`;
};

const svgoConfig = {
  plugins: ["preset-default"],
};

const compressAndWrite = (filename, content) => {
  const result = optimize(content, svgoConfig);
  writeFileSync(filename, result.data);
};

compressAndWrite("svg/en.svg", combine(data, "en"));
compressAndWrite("svg/zh.svg", combine(data, "zh"));
