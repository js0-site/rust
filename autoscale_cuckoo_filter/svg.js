#!/usr/bin/env bun

import { readFileSync, writeFileSync } from 'fs';
import { optimize } from 'svgo';

const data = JSON.parse(readFileSync('bench.json', 'utf-8'));

// 颜色 Colors
const COLORS = ['#5470c6', '#91cc75', '#fac858', '#ee6666', '#73c0de', '#3ba272', '#fc8452', '#9a60b4'];
const DEPTH = 12, ANGLE = 0.6;

const lighten = (hex, p) => {
  const n = parseInt(hex.slice(1), 16);
  const r = Math.min(255, (n >> 16) + (2.55 * p) | 0);
  const g = Math.min(255, ((n >> 8) & 0xFF) + (2.55 * p) | 0);
  const b = Math.min(255, (n & 0xFF) + (2.55 * p) | 0);
  return `#${(r << 16 | g << 8 | b).toString(16).padStart(6, '0')}`;
};

const darken = (hex, p) => {
  const n = parseInt(hex.slice(1), 16);
  const r = Math.max(0, (n >> 16) - (2.55 * p) | 0);
  const g = Math.max(0, ((n >> 8) & 0xFF) - (2.55 * p) | 0);
  const b = Math.max(0, (n & 0xFF) - (2.55 * p) | 0);
  return `#${(r << 16 | g << 8 | b).toString(16).padStart(6, '0')}`;
};

// 3D柱 3D bar
const bar3d = (x, y, w, h, c) => `<g>
  <path d="M${x+w},${y} L${x+w+DEPTH*ANGLE},${y-DEPTH} L${x+w+DEPTH*ANGLE},${y+h-DEPTH} L${x+w},${y+h} Z" fill="${darken(c,20)}"/>
  <path d="M${x},${y} L${x+DEPTH*ANGLE},${y-DEPTH} L${x+w+DEPTH*ANGLE},${y-DEPTH} L${x+w},${y} Z" fill="${lighten(c,25)}"/>
  <rect x="${x}" y="${y}" width="${w}" height="${h}" fill="${c}" rx="1"/>
</g>`;

// 性能图 Performance chart
const perfChart = (results, lang) => {
  const ops = lang === 'en' ? ['Contains', 'Add', 'Remove'] : ['查询', '添加', '删除'];
  const title = lang === 'en' ? 'Performance (M ops/s)' : '性能 (百万次/秒)';
  const W = 580, H = 260, m = { t: 45, r: 25, b: 70, l: 55 };
  const cW = W - m.l - m.r, cH = H - m.t - m.b;
  
  let max = 0;
  for (const r of results) max = Math.max(max, r.contains_mops, r.add_mops, r.remove_mops);
  max = Math.ceil(max / 20) * 20;
  
  const libs = results.map(r => r.lib);
  const gW = cW / 3, bW = Math.min(35, (gW - 20) / libs.length), gap = 5;
  
  let svg = `<text x="${W/2}" y="28" text-anchor="middle" font-size="14" font-weight="bold" fill="#333">${title}</text>`;
  
  // 网格 Grid
  for (let i = 0; i <= 5; i++) {
    const y = m.t + cH - cH * i / 5;
    svg += `<line x1="${m.l}" y1="${y}" x2="${W-m.r}" y2="${y}" stroke="#e0e0e0"/>`;
    svg += `<text x="${m.l-8}" y="${y+4}" text-anchor="end" font-size="10" fill="#666">${max*i/5|0}</text>`;
  }
  svg += `<line x1="${m.l}" y1="${m.t+cH}" x2="${W-m.r}" y2="${m.t+cH}" stroke="#888"/>`;
  svg += `<line x1="${m.l}" y1="${m.t}" x2="${m.l}" y2="${m.t+cH}" stroke="#888"/>`;
  
  // 柱状图 Bars
  const keys = ['contains_mops', 'add_mops', 'remove_mops'];
  ops.forEach((op, oi) => {
    const gX = m.l + gW * oi + (gW - libs.length * (bW + gap)) / 2;
    svg += `<text x="${m.l + gW * oi + gW/2}" y="${H-m.b+20}" text-anchor="middle" font-size="11" fill="#333">${op}</text>`;
    results.forEach((r, li) => {
      const v = r[keys[oi]], h = (v / max) * cH;
      const x = gX + li * (bW + gap), y = m.t + cH - h;
      svg += bar3d(x, y, bW, h, COLORS[li % COLORS.length]);
      svg += `<text x="${x+bW/2}" y="${y-DEPTH-3}" text-anchor="middle" font-size="9" fill="#333">${v.toFixed(1)}</text>`;
    });
  });
  
  return svg;
};


// 内存图 Memory chart
const memChart = (results, lang) => {
  const title = lang === 'en' ? 'Memory (KB)' : '内存 (KB)';
  const W = 260, H = 200, m = { t: 40, r: 15, b: 60, l: 50 };
  const cW = W - m.l - m.r, cH = H - m.t - m.b;
  
  let max = Math.max(...results.map(r => r.memory_kb));
  max = Math.ceil(max / 100) * 100 || 100;
  
  const bW = Math.min(45, (cW - 15) / results.length), gap = 12;
  const total = results.length * bW + (results.length - 1) * gap;
  const startX = m.l + (cW - total) / 2;
  
  let svg = `<text x="${W/2}" y="22" text-anchor="middle" font-size="12" font-weight="bold" fill="#333">${title}</text>`;
  
  for (let i = 0; i <= 4; i++) {
    const y = m.t + cH - cH * i / 4;
    svg += `<line x1="${m.l}" y1="${y}" x2="${W-m.r}" y2="${y}" stroke="#e0e0e0"/>`;
    svg += `<text x="${m.l-6}" y="${y+3}" text-anchor="end" font-size="9" fill="#666">${max*i/4|0}</text>`;
  }
  svg += `<line x1="${m.l}" y1="${m.t+cH}" x2="${W-m.r}" y2="${m.t+cH}" stroke="#888"/>`;
  
  results.forEach((r, i) => {
    const h = (r.memory_kb / max) * cH;
    const x = startX + i * (bW + gap), y = m.t + cH - h;
    svg += bar3d(x, y, bW, h, COLORS[i % COLORS.length]);
    svg += `<text x="${x+bW/2}" y="${y-DEPTH-3}" text-anchor="middle" font-size="9" fill="#333">${Math.round(r.memory_kb)}</text>`;
    const lx = x + bW / 2, ly = m.t + cH + 8;
    const name = r.lib.length > 12 ? r.lib.slice(0, 12) + '…' : r.lib;
    svg += `<text x="${lx}" y="${ly}" text-anchor="end" font-size="8" fill="#333" transform="rotate(-40 ${lx} ${ly})">${name}</text>`;
  });
  
  return svg;
};

// FPP图 FPP chart
const fppChart = (results, lang) => {
  const title = lang === 'en' ? 'FPP (%)' : '误判率 (%)';
  const W = 260, H = 200, m = { t: 40, r: 15, b: 60, l: 50 };
  const cW = W - m.l - m.r, cH = H - m.t - m.b;
  
  const fpps = results.map(r => r.fpp * 100);
  let max = Math.max(...fpps);
  max = Math.ceil(max * 10) / 10 || 0.5;
  if (max < 0.2) max = 0.2;
  
  const bW = Math.min(45, (cW - 15) / results.length), gap = 12;
  const total = results.length * bW + (results.length - 1) * gap;
  const startX = m.l + (cW - total) / 2;
  
  let svg = `<text x="${W/2}" y="22" text-anchor="middle" font-size="12" font-weight="bold" fill="#333">${title}</text>`;
  
  for (let i = 0; i <= 4; i++) {
    const y = m.t + cH - cH * i / 4;
    svg += `<line x1="${m.l}" y1="${y}" x2="${W-m.r}" y2="${y}" stroke="#e0e0e0"/>`;
    svg += `<text x="${m.l-6}" y="${y+3}" text-anchor="end" font-size="9" fill="#666">${(max*i/4).toFixed(2)}</text>`;
  }
  svg += `<line x1="${m.l}" y1="${m.t+cH}" x2="${W-m.r}" y2="${m.t+cH}" stroke="#888"/>`;
  
  results.forEach((r, i) => {
    const v = r.fpp * 100, h = (v / max) * cH;
    const x = startX + i * (bW + gap), y = m.t + cH - h;
    svg += bar3d(x, y, bW, h, COLORS[i % COLORS.length]);
    svg += `<text x="${x+bW/2}" y="${y-DEPTH-3}" text-anchor="middle" font-size="9" fill="#333">${v.toFixed(2)}</text>`;
    const lx = x + bW / 2, ly = m.t + cH + 8;
    const name = r.lib.length > 12 ? r.lib.slice(0, 12) + '…' : r.lib;
    svg += `<text x="${lx}" y="${ly}" text-anchor="end" font-size="8" fill="#333" transform="rotate(-40 ${lx} ${ly})">${name}</text>`;
  });
  
  return svg;
};

// 组合 Combine
const combine = (results, lang) => {
  const W = 580, H = 500;
  const perf = perfChart(results, lang);
  const mem = memChart(results, lang);
  const fpp = fppChart(results, lang);
  
  // 图例 Legend
  const libs = results.map(r => r.lib);
  let legend = '';
  libs.forEach((lib, i) => {
    const x = 55 + i * 175;
    legend += `<rect x="${x}" y="0" width="12" height="12" fill="${COLORS[i%COLORS.length]}" rx="2"/>`;
    legend += `<text x="${x+16}" y="10" font-size="10" fill="#333">${lib}</text>`;
  });
  
  return `<svg xmlns="http://www.w3.org/2000/svg" width="${W}" height="${H}" viewBox="0 0 ${W} ${H}">
<rect width="${W}" height="${H}" fill="#fff"/>
<g>${perf}</g>
<g transform="translate(15,265)">${mem}</g>
<g transform="translate(295,265)">${fpp}</g>
<g transform="translate(0,478)">${legend}</g>
</svg>`;
};

// 生成 Generate
const { results } = data;

// SVG压缩配置 SVG compression config
const svgoConfig = {
  plugins: [
    'preset-default',
    {
      name: 'removeViewBox',
      active: false
    },
    {
      name: 'removeDimensions',
      active: false
    }
  ]
};

const compressAndWrite = (filename, content) => {
  const result = optimize(content, svgoConfig);
  writeFileSync(filename, result.data);
};

compressAndWrite('readme/en.svg', combine(results, 'en'));
compressAndWrite('readme/zh.svg', combine(results, 'zh'));
console.log('Done: readme/en.svg, readme/zh.svg (compressed)');
