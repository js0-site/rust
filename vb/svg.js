#!/usr/bin/env bun

import { readFileSync, writeFileSync, mkdirSync } from "fs";
import { optimize } from "svgo";

const data = JSON.parse(readFileSync("bench.json", "utf-8"));

// Modern vibrant color palette
const LIB_COLORS = {
  vb: "#6366f1", // Indigo
  "integer-encoding": "#14b8a6", // Teal
  leb128: "#f97316", // Orange
};
const DEFAULT_COLOR = "#8b5cf6";
const MAIN_TITLE_SIZE = 32;

const getColor = (lib) => LIB_COLORS[lib] || DEFAULT_COLOR;

// Lighten color for top face
const lighten = (hex, amount = 0.2) => {
  const num = parseInt(hex.slice(1), 16);
  const r = Math.min(
    255,
    ((num >> 16) & 0xff) + (255 - ((num >> 16) & 0xff)) * amount,
  );
  const g = Math.min(
    255,
    ((num >> 8) & 0xff) + (255 - ((num >> 8) & 0xff)) * amount,
  );
  const b = Math.min(255, (num & 0xff) + (255 - (num & 0xff)) * amount);
  return `rgb(${Math.round(r)},${Math.round(g)},${Math.round(b)})`;
};

// Darken color for side face
const darken = (hex, amount = 0.25) => {
  const num = parseInt(hex.slice(1), 16);
  const r = Math.max(0, ((num >> 16) & 0xff) * (1 - amount));
  const g = Math.max(0, ((num >> 8) & 0xff) * (1 - amount));
  const b = Math.max(0, (num & 0xff) * (1 - amount));
  return `rgb(${Math.round(r)},${Math.round(g)},${Math.round(b)})`;
};

// 3D bar with gradient-like effect
const bar3d = (x, y, w, h, color, depth = 10) => {
  const light = lighten(color, 0.3);
  const dark = darken(color, 0.3);

  let svg = "";
  // Front face with subtle gradient effect
  svg += `<rect x="${x}" y="${y}" width="${w}" height="${h}" fill="${color}"/>`;
  // Highlight on front face
  svg += `<rect x="${x}" y="${y}" width="${w * 0.15}" height="${h}" fill="${light}" opacity="0.4"/>`;
  // Top face
  svg += `<polygon points="${x},${y} ${x + depth},${y - depth} ${x + w + depth},${y - depth} ${x + w},${y}" fill="${light}"/>`;
  // Right face
  svg += `<polygon points="${x + w},${y} ${x + w + depth},${y - depth} ${x + w + depth},${y + h - depth} ${x + w},${y + h}" fill="${dark}"/>`;

  return svg;
};

const barChart = (results, m, yOffset, getValue, formatLabel) => {
  const cH = m.chartH - m.t - m.b;
  const depth = 10;
  let maxVal = 0;
  for (const r of results) {
    maxVal = Math.max(maxVal, getValue(r));
  }

  const bW = Math.min(65, (m.cW - 40) / results.length - 25);
  const gap = (m.cW - results.length * bW) / (results.length + 1);

  let svg = "";

  // Grid lines with softer color
  for (let i = 0; i <= 4; i++) {
    const y = yOffset + m.t + cH - (cH * i) / 4;
    svg += `<line x1="${m.l}" y1="${y}" x2="${m.l + m.cW}" y2="${y}" stroke="#e5e7eb" stroke-dasharray="${i === 0 ? "" : "4,4"}"/>`;
    const label = formatLabel((maxVal * i) / 4);
    svg += `<text x="${m.l - 12}" y="${y + 5}" text-anchor="end" font-size="12" fill="#9ca3af">${label}</text>`;
  }

  const bestVal = Math.max(...results.map(getValue));

  results.forEach((r, i) => {
    const x = m.l + gap + i * (bW + gap);
    const v = getValue(r);
    const h = (v / maxVal) * cH;
    const y = yOffset + m.t + cH - h;
    const isBest = v === bestVal;
    const color = getColor(r.lib);

    svg += bar3d(x, y, bW, h, color, depth);

    // Value label with better styling
    svg += `<text x="${x + bW / 2 + depth / 2}" y="${y - depth - 8}" text-anchor="middle" font-size="13" font-weight="600" fill="${isBest ? "#059669" : "#374151"}">${formatLabel(v)}</text>`;

    // Library name
    const labelY = yOffset + m.chartH - m.b + 22;
    svg += `<text x="${x + bW / 2}" y="${labelY}" text-anchor="middle" font-size="13" font-weight="500" fill="#4b5563">${r.lib}</text>`;
  });

  return svg;
};

const combine = (data, lang) => {
  const { results, data_count } = data;
  const libs = results.map((r) => r.lib);

  const m = { t: 70, r: 50, b: 55, l: 70, cW: 400, chartH: 280 };
  const W = m.l + m.cW + m.r;
  const chartGap = 15;
  const titleH = 85;

  const mainTitle =
    lang === "en" ? "VByte Encoding Benchmark" : "VByte 编码性能评测";
  const subTitle =
    lang === "en"
      ? `${data_count.toLocaleString()} integers · mixed distribution`
      : `${data_count.toLocaleString()} 个整数 · 混合分布`;

  const titles =
    lang === "en"
      ? ["Encode Throughput (M/s)", "Decode Throughput (M/s)"]
      : ["编码吞吐量 (百万/秒)", "解码吞吐量 (百万/秒)"];

  const resultsWithThroughput = results.map((r) => ({
    ...r,
    encode_throughput: ((data_count / r.encode_ns) * 1e9) / 1e6,
    decode_throughput: ((data_count / r.decode_ns) * 1e9) / 1e6,
  }));

  const sortedByEncode = [...resultsWithThroughput].sort(
    (a, b) => b.encode_throughput - a.encode_throughput,
  );
  const sortedByDecode = [...resultsWithThroughput].sort(
    (a, b) => b.decode_throughput - a.decode_throughput,
  );

  const charts = [
    {
      data: sortedByEncode,
      getValue: (r) => r.encode_throughput,
      format: (v) => v.toFixed(1),
    },
    {
      data: sortedByDecode,
      getValue: (r) => r.decode_throughput,
      format: (v) => v.toFixed(1),
    },
  ];

  // Background transparent
  let svg = "";

  // Title
  svg += `<text x="${W / 2}" y="42" text-anchor="middle" font-size="${MAIN_TITLE_SIZE}" font-weight="700" fill="#111827">${mainTitle}</text>`;
  svg += `<text x="${W / 2}" y="68" text-anchor="middle" font-size="14" fill="#6b7280">${subTitle}</text>`;
  let yOffset = titleH;

  charts.forEach((chart, idx) => {
    svg += `<text x="${W / 2}" y="${yOffset + 30}" text-anchor="middle" font-size="16" font-weight="600" fill="#374151">${titles[idx]}</text>`;
    svg += barChart(chart.data, m, yOffset, chart.getValue, chart.format);
    yOffset += m.chartH + chartGap;
  });

  const H = yOffset - 10;

  return `<svg xmlns="http://www.w3.org/2000/svg" width="${W}" height="${H}" viewBox="0 0 ${W} ${H}">
${svg}
</svg>`;
};

try {
  mkdirSync("svg", { recursive: true });
} catch {}

const svgoConfig = { plugins: ["preset-default"] };
const compressAndWrite = (filename, content) => {
  const result = optimize(content, svgoConfig);
  writeFileSync(filename, result.data);
};

compressAndWrite("svg/en.svg", combine(data, "en"));
compressAndWrite("svg/zh.svg", combine(data, "zh"));
console.log("Generated: svg/en.svg, svg/zh.svg");
