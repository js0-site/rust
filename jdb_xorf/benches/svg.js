#!/usr/bin/env node
import { writeFileSync, mkdirSync } from "fs";
import { getTargetDir, parseCriterion, compareFilters, normalizeFilter } from "./lib.js";
import { RESOURCES } from "./i18n/index.js";

const LANGS = Object.entries(RESOURCES).map(([code, labels]) => ({
  code,
  labels,
  titles: {
    main: labels.svg_title_main,
    build: labels.svg_title_build,
    query: labels.svg_title_query,
    memory: labels.svg_title_memory,
    accuracy: labels.svg_title_accuracy,
  },
}));

const COLORS = {
  jdb: "#DC2626", // 橙红色 (red-600)
  xorf: "#1E40AF", // 深蓝色 (blue-800)
};

const genSvg = (results, titles, labels) => {
  const width = 1000;
  const height = 1400;
  const padding = 40;

  // Four charts vertically stacked
  // 四个图表垂直堆叠
  const chartWidth = width - padding * 2;
  const chartHeight = (height - padding * 5) / 4;

  let svg = `<svg width="${width}" height="${height}" xmlns="http://www.w3.org/2000/svg">`;
  svg += `<rect width="${width}" height="${height}" fill="white"/>`;

  // Main title
  // 主标题
  svg += `<text x="${width / 2}" y="30" text-anchor="middle" font-size="20" font-weight="bold">${titles.main}</text>`;

  const filters = [];
  for (const [lib, libFilters] of Object.entries(results)) {
    for (const [filter, sizes] of Object.entries(libFilters)) {
      for (const [size, metrics] of Object.entries(sizes)) {
        const sizeNum = parseInt(size) || 100000;
        const buildTime = metrics.build?.mean_ns || 0;
        const queryTime = metrics.has?.mean_ns || 0;

        const normalizedFilter = normalizeFilter(filter);
        filters.push({
          lib,
          filter: normalizedFilter,
          name: `${lib}_${normalizedFilter}`,
          // Throughput calculation: (size * 1000) / mean_ns = Mops/s
          build: buildTime > 0 ? (sizeNum * 1000) / buildTime : 0,
          query: queryTime > 0 ? (sizeNum * 1000) / queryTime : 0,
          memory: metrics.memory?.bytes || 0,
          fpRate: metrics.false_positive?.rate || 0,
        });
      }
    }
  }

  // Sort by filter type (8, 16, 32) then by lib name
  // 按过滤器类型（8, 16, 32）然后按库名排序
  filters.sort((a, b) => {
    const cmp = compareFilters(a.filter, b.filter);
    if (cmp !== 0) return cmp;
    return a.lib.localeCompare(b.lib);
  });

  if (filters.length === 0) {
    svg += `<text x="${width / 2}" y="${height / 2}" text-anchor="middle" font-size="16">${labels.svg_no_data}</text>`;
    svg += `</svg>`;
    return svg;
  }

  // Chart 1: Query Throughput
  // 图表1：查询吞吐量
  svg += genBarChart(
    filters,
    padding,
    padding + 50,
    chartWidth,
    chartHeight,
    titles.query,
    "query",
    "Mops/s",
    1,
    labels,
  );

  // Chart 2: Bf Throughput
  // 图表2：构建吞吐量
  svg += genBarChart(
    filters,
    padding,
    padding + 50 + chartHeight + padding,
    chartWidth,
    chartHeight,
    titles.build,
    "build",
    "Mops/s",
    1,
    labels,
  );

  // Chart 3: Memory
  // 图表3：内存
  svg += genBarChart(
    filters,
    padding,
    padding + 50 + (chartHeight + padding) * 2,
    chartWidth,
    chartHeight,
    titles.memory,
    "memory",
    "KB",
    1024,
    labels,
  );

  // Chart 4: Accuracy (FPR)
  // 图表4：准确率 (FPR)
  svg += genBarChart(
    filters,
    padding,
    padding + 50 + (chartHeight + padding) * 3,
    chartWidth,
    chartHeight,
    titles.accuracy,
    "fpRate",
    "%",
    1,
    labels,
  );

  svg += `</svg>`;
  return svg;
};

const genBarChart = (filters, x, y, w, h, title, metric, unit, divisor, labels) => {
  let svg = `<g transform="translate(${x}, ${y})">`;

  // Title
  // 标题
  svg += `<text x="${w / 2}" y="0" text-anchor="middle" font-size="16" font-weight="bold">${title}</text>`;

  const maxValue = Math.max(...filters.map((f) => f[metric]), 1);
  const barWidth = w / (filters.length * 1.5);
  const chartH = h - 80;

  // Axis
  // 坐标轴
  svg += `<line x1="0" y1="30" x2="0" y2="${chartH + 30}" stroke="#333" stroke-width="2"/>`;
  svg += `<line x1="0" y1="${chartH + 30}" x2="${w}" y2="${chartH + 30}" stroke="#333" stroke-width="2"/>`;
  svg += `<text x="-5" y="25" text-anchor="end" font-size="12">${unit}</text>`;

  // Group by filter type for comparison
  // 按过滤器类型分组以便对比
  const filterTypes = ["Bf8", "Bf16", "Bf32"];
  const libs = [...new Set(filters.map((f) => f.lib))].sort();

  let groupX = 20;
  const groupWidth = (w - 40) / filterTypes.length;
  const barWidthInGroup = groupWidth / (libs.length + 1);

  filterTypes.forEach((filterType, typeIdx) => {
    const groupFilters = filters.filter((f) => f.filter === filterType);

    // Find best performer in this group
    // 找到该组中性能最好的
    let bestFilter = null;
    if (groupFilters.length > 0) {
      if (metric === "build" || metric === "query") {
        // For throughput, higher is better
        // 吞吐量越高越好
        bestFilter = groupFilters.reduce((best, curr) =>
          curr[metric] > best[metric] ? curr : best,
        );
      } else {
        // For memory and FPR, lower is better
        // 内存和误报率越低越好
        bestFilter = groupFilters.reduce((best, curr) =>
          curr[metric] < best[metric] ? curr : best,
        );
      }
    }

    groupFilters.forEach((filter, idx) => {
      const bx = groupX + idx * barWidthInGroup;
      const value = filter[metric];
      const barH = (value / maxValue) * chartH;
      const color = COLORS[filter.lib];
      const isBest =
        bestFilter &&
        filter.lib === bestFilter.lib &&
        filter.filter === bestFilter.filter;

      // Check if there are different values in the group
      // 检查组内是否有不同的值
      const hasDistinctValues =
        groupFilters.length > 1 &&
        groupFilters.some((f) => f[metric] !== bestFilter[metric]);

      // Bar
      // 柱状图
      svg += `<rect x="${bx}" y="${chartH + 30 - barH}" width="${barWidthInGroup * 0.85}" height="${barH}" fill="${color}" rx="2"/>`;

      // Value label
      // 数值标签
      const displayValue =
        metric === "fpRate"
          ? value.toFixed(3)
          : (value / divisor).toFixed(metric === "memory" ? 1 : 0);
      svg += `<text x="${bx + barWidthInGroup * 0.425}" y="${chartH + 30 - barH - 3}" text-anchor="middle" font-size="10" font-weight="bold">${displayValue} ${unit}</text>`;

      // Star for best performer (only if values are different)
      // 性能最好的标注五角星（仅当值不同时）
      if (isBest && hasDistinctValues) {
        const starX = bx + barWidthInGroup * 0.425;
        const starY = chartH + 30 - barH - 38;
        svg += `<path d="M ${starX} ${starY} L ${starX + 4} ${starY + 8} L ${starX + 12} ${starY + 8} L ${starX + 6} ${starY + 13} L ${starX + 8} ${starY + 21} L ${starX} ${starY + 16} L ${starX - 8} ${starY + 21} L ${starX - 6} ${starY + 13} L ${starX - 12} ${starY + 8} L ${starX - 4} ${starY + 8} Z" fill="#EF4444" stroke="#DC2626" stroke-width="0.5"/>`;
      }

      // Lib name label on bar
      // 柱子上的库名标签
      if (barH > 20) {
        svg += `<text x="${bx + barWidthInGroup * 0.425}" y="${chartH + 30 - barH / 2}" text-anchor="middle" font-size="9" fill="white" font-weight="bold">${filter.lib}</text>`;
      }
    });

    // Filter type label
    // 过滤器类型标签
    svg += `<text x="${groupX + groupWidth / 2}" y="${chartH + 50}" text-anchor="middle" font-size="12" font-weight="bold">${filterType}</text>`;

    groupX += groupWidth;
  });

  // Legend
  // 图例
  const legendX = w - 100;
  const legendY = 40;
  libs.forEach((lib, idx) => {
    const ly = legendY + idx * 20;
    svg += `<rect x="${legendX}" y="${ly}" width="15" height="15" fill="${COLORS[lib]}" rx="2"/>`;
    svg += `<text x="${legendX + 20}" y="${ly + 12}" font-size="11" font-weight="bold">${lib}</text>`;
  });

  // Star legend
  // 五角星图例
  const starLegendY = legendY + libs.length * 20 + 10;
  svg += `<path d="M ${legendX + 7.5} ${starLegendY} L ${legendX + 9.5} ${starLegendY + 4} L ${legendX + 13.5} ${starLegendY + 4} L ${legendX + 10.5} ${starLegendY + 6.5} L ${legendX + 11.5} ${starLegendY + 10.5} L ${legendX + 7.5} ${starLegendY + 8} L ${legendX + 3.5} ${starLegendY + 10.5} L ${legendX + 4.5} ${starLegendY + 6.5} L ${legendX + 1.5} ${starLegendY + 4} L ${legendX + 5.5} ${starLegendY + 4} Z" fill="#EF4444" stroke="#DC2626" stroke-width="0.5"/>`;
  svg += `<text x="${legendX + 20}" y="${starLegendY + 8}" font-size="10">${labels.svg_best}</text>`;

  svg += `</g>`;
  return svg;
};

const main = async () => {
  const target_dir = await getTargetDir();
  const results = parseCriterion(target_dir);

  mkdirSync("readme", { recursive: true });

  await Promise.all(
    LANGS.map(({ code, titles, labels }) => {
      const svg = genSvg(results, titles, labels);
      const path = `readme/${code}.bench.svg`;
      writeFileSync(path, svg);
      console.log(`  ${path}`);
    }),
  );

  console.log("Generated benchmark SVG charts");
};

main();
