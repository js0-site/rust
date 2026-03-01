#!/usr/bin/env bun

import { readFileSync, writeFileSync } from "fs";
import { optimize } from "svgo";

const readBench = (name) => {
  try {
    return JSON.parse(readFileSync(`bench/${name}.json`, "utf-8"));
  } catch (e) {
    console.warn(`Failed to read bench/${name}.json`, e);
    return { first_resolution: 0, repeated_resolution: 0 };
  }
};

const results = [
  {
    lib: "compio-net",
    ...readBench("compio-net "),
  },
  {
    lib: "compio-net + compio_dns",
    ...readBench("compio-net + compio_dns"),
  },
];

// 颜色 Colors
// 颜色 Colors
const COLORS = [
  "#5470c6",
  "#ff4400", // Sexy Orange Red
  "#fac858",
  "#91cc75",
  "#73c0de",
  "#3ba272",
  "#fc8452",
  "#9a60b4",
];
const DEPTH = 12,
  ANGLE = 0.6;

const lighten = (hex, p) => {
  const n = parseInt(hex.slice(1), 16);
  const r = Math.min(255, ((n >> 16) + 2.55 * p) | 0);
  const g = Math.min(255, (((n >> 8) & 0xff) + 2.55 * p) | 0);
  const b = Math.min(255, ((n & 0xff) + 2.55 * p) | 0);
  return `#${((r << 16) | (g << 8) | b).toString(16).padStart(6, "0")}`;
};

const darken = (hex, p) => {
  const n = parseInt(hex.slice(1), 16);
  const r = Math.max(0, ((n >> 16) - 2.55 * p) | 0);
  const g = Math.max(0, (((n >> 8) & 0xff) - 2.55 * p) | 0);
  const b = Math.max(0, ((n & 0xff) - 2.55 * p) | 0);
  return `#${((r << 16) | (g << 8) | b).toString(16).padStart(6, "0")}`;
};

// 3D柱 3D bar
const bar3d = (x, y, w, h, c) => `<g>
  <path d="M${x + w},${y} L${x + w + DEPTH * ANGLE},${y - DEPTH} L${x + w + DEPTH * ANGLE},${y + h - DEPTH} L${x + w},${y + h} Z" fill="${darken(c, 20)}"/>
  <path d="M${x},${y} L${x + DEPTH * ANGLE},${y - DEPTH} L${x + w + DEPTH * ANGLE},${y - DEPTH} L${x + w},${y} Z" fill="${lighten(c, 25)}"/>
  <rect x="${x}" y="${y}" width="${w}" height="${h}" fill="${c}" rx="1"/>
</g>`;

// 单个图表 Single chart
const singleChart = (results, key, title, xOffset, lang) => {
  const W = 280, // Half width
    H = 360,
    m = { t: 80, r: 15, b: 60, l: 65 }; // Increased top margin, adjusted bottom
  const cW = W - m.l - m.r,
    cH = H - m.t - m.b;

  let max = 0;
  for (const r of results) max = Math.max(max, r[key]);

  // Round up magnitude
  const magnitude = Math.pow(10, Math.floor(Math.log10(max)));
  max = Math.ceil(max / magnitude) * magnitude;
  if (max === 0) max = 100;

  const libs = results.map((r) => r.lib);
  // Bar width calculation
  const bW = Math.min(40, (cW - 20) / libs.length),
    gap = 10;
  const totalBarW = libs.length * (bW + gap) - gap;
  const gX = xOffset + m.l + (cW - totalBarW) / 2;

  let svg = `<text x="${xOffset + W / 2}" y="35" text-anchor="middle" font-size="14" font-weight="bold" fill="#333">${title}</text>`;

  // Grid
  for (let i = 0; i <= 5; i++) {
    const y = m.t + cH - (cH * i) / 5;
    svg += `<line x1="${xOffset + m.l}" y1="${y}" x2="${xOffset + W - m.r}" y2="${y}" stroke="#e0e0e0"/>`;
    svg += `<text x="${xOffset + m.l - 8}" y="${y + 4}" text-anchor="end" font-size="10" fill="#666">${((max * i) / 5) | 0}</text>`;
  }
  svg += `<line x1="${xOffset + m.l}" y1="${m.t + cH}" x2="${xOffset + W - m.r}" y2="${m.t + cH}" stroke="#888"/>`;
  svg += `<line x1="${xOffset + m.l}" y1="${m.t}" x2="${xOffset + m.l}" y2="${m.t + cH}" stroke="#888"/>`;

  // Bars
  results.forEach((r, li) => {
    const v = r[key],
      h = (v / max) * cH;
    const x = gX + li * (bW + gap),
      y = m.t + cH - h;

    const barH = Math.max(1, h);
    const barY = m.t + cH - barH;

    svg += bar3d(x, barY, bW, barH, COLORS[li % COLORS.length]);
    svg += `<text x="${x + bW / 2}" y="${barY - DEPTH - 3}" text-anchor="middle" font-size="10" fill="#333">${v}</text>`;

    // Speedup text for the second bar
    if (li === 1 && results.length >= 2 && results[0][key] > 0) {
      const ratio = v / results[0][key];
      const ratioStr = ratio.toFixed(1);
      const speedup = lang === "en" ? `x${ratioStr}` : `提速 ${ratioStr}x`;
      svg += `<text x="${x + bW / 2}" y="${barY - DEPTH - 18}" text-anchor="middle" font-size="11" font-weight="bold" fill="#ff4400">${speedup}</text>`;
    }
  });

  return svg;
};

// 组合 Combine
const combine = (results, lang) => {
  const W = 600, // Total width
    H = 380;

  const title1 = lang === "en" ? "First (RPS)" : "首次解析 (次/秒)";
  const title2 = lang === "en" ? "Repeated (RPS)" : "重复解析 (次/秒)";

  const chart1 = singleChart(results, "first_resolution", title1, 0, lang);
  const chart2 = singleChart(results, "repeated_resolution", title2, 300, lang); // Right offset 300

  // 图例 Legend
  const libs = results.map((r) => r.lib);
  let legend = "";
  const totalLegendWidth = libs.reduce((acc, lib) => acc + 20 + lib.length * 6 + 20, 0);
  let currentX = (W - totalLegendWidth) / 2;

  libs.forEach((lib, i) => {
    const textWidth = lib.length * 7;
    const itemWidth = 20 + textWidth + 20;

    legend += `<rect x="${currentX}" y="0" width="12" height="12" fill="${COLORS[i % COLORS.length]}" rx="2"/>`;
    legend += `<text x="${currentX + 16}" y="10" font-size="11" fill="#333">${lib}</text>`;
    currentX += itemWidth;
  });

  return `<svg xmlns="http://www.w3.org/2000/svg" width="${W}" height="${H}" viewBox="0 0 ${W} ${H}">
<rect width="${W}" height="${H}" fill="#fff"/>
<g>${chart1}</g>
<g>${chart2}</g>
<g transform="translate(0,345)">${legend}</g>
</svg>`;
};

// SVG压缩配置 SVG compression config
const svgoConfig = {
  plugins: [
    "preset-default",
    {
      name: "removeViewBox",
      active: false,
    },
    {
      name: "removeDimensions",
      active: false,
    },
  ],
};

const compressAndWrite = (filename, content) => {
  const result = optimize(content, svgoConfig);
  writeFileSync(filename, result.data);
};

compressAndWrite("readme/en.svg", combine(results, "en"));
compressAndWrite("readme/zh.svg", combine(results, "zh"));
console.log("✅");
