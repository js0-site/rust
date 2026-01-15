#!/usr/bin/env bun

// SVG chart generator for Pgm-Index benchmark
// Pgm-Index 基准测试 SVG 图表生成器

import { readFileSync, writeFileSync } from "fs";
import { join } from "path";
import {
  ALGORITHM_COLORS,
  ALGORITHM_NAMES,
  ALGORITHM_NAMES_ZH,
  getColor,
  formatDataSize,
  formatMemory,
} from "./js/common.js";

const ROOT = import.meta.dirname;
const BENCH_PATH = join(ROOT, "bench.json");
const ACCURACY_PATH = join(ROOT, "accuracy.json");
const EN_SVG = join(ROOT, "svg/en.svg");
const ZH_SVG = join(ROOT, "svg/zh.svg");

// Chart dimensions
// 图表尺寸
const W = 700;
const CHART_H = 200;
const M = { t: 150, r: 40, b: 80, l: 90 };
const BAR_W = 60;
const BAR_GAP = 45;

// Escape XML special characters
// 转义 XML 特殊字符
const escXml = (s) =>
  String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");

// Generate Y-axis with grid lines
// 生成 Y 轴和网格线
const genYAxis = (baseY, h, maxVal, unit, labelX = 80) => {
  const ticks = 5;
  let svg = "";
  for (let i = 0; i <= ticks; i++) {
    const y = baseY + h - (i / ticks) * h;
    const val = ((i / ticks) * maxVal).toFixed(2);
    svg += `<path stroke="#e0e0e0" d="M${M.l} ${y}h${W - M.l - M.r}"/>`;
    svg += `<text x="${labelX}" y="${y + 6}" fill="#666" font-size="16" text-anchor="end">${val}</text>`;
  }
  svg += `<path stroke="#888" d="M${M.l} ${baseY + h}h${W - M.l - M.r}"/>`;
  return svg;
};

// Generate vertical axis label
// 生成垂直轴标签
const genYLabel = (baseY, h, label) => {
  const cy = baseY + h / 2;
  return `<text x="20" y="${cy}" fill="#333" font-size="16" font-weight="bold" text-anchor="middle" transform="rotate(-90 20 ${cy})">${escXml(label)}</text>`;
};

// Generate bar with value label
// 生成带数值标签的柱状图
const genBar = (x, baseY, h, val, maxVal, color, label, subLabel = null) => {
  const barH = maxVal > 0 ? (val / maxVal) * h : 0;
  const y = baseY + h - barH;
  let svg = "";
  if (barH > 0) {
    svg += `<path fill="${color}" d="M${x} ${y}h${BAR_W}v${barH}h-${BAR_W}z"/>`;
  }
  svg += `<text x="${x + BAR_W / 2}" y="${y - 8}" fill="#333" font-size="12" text-anchor="middle">${val.toFixed(2)}</text>`;
  svg += `<text x="${x + BAR_W / 2}" y="${baseY + h + 20}" fill="#333" font-size="14" text-anchor="end" transform="rotate(-45 ${x + BAR_W / 2} ${baseY + h + 20})">${escXml(label)}</text>`;
  if (subLabel) {
    svg += `<text x="${x + BAR_W / 2}" y="${baseY + h + 40}" fill="#666" font-size="12" text-anchor="end" transform="rotate(-45 ${x + BAR_W / 2} ${baseY + h + 40})">${escXml(subLabel)}</text>`;
  }
  return svg;
};

// Generate legend
// 生成图例
const genLegend = (y, names) => {
  const items = [
    { key: "jdb_pgm", color: ALGORITHM_COLORS.jdb_pgm },
    { key: "external_pgm", color: ALGORITHM_COLORS.external_pgm },
    { key: "hashmap", color: ALGORITHM_COLORS.hashmap },
    { key: "binary_search", color: ALGORITHM_COLORS.binary_search },
    { key: "btreemap", color: ALGORITHM_COLORS.btreemap },
  ];
  let svg = "";
  const cols = 3;
  const colW = 190;
  items.forEach((item, i) => {
    const col = i % cols;
    const row = Math.floor(i / cols);
    const x = M.l + col * colW;
    const ly = y + row * 32;
    svg += `<path fill="${item.color}" d="M${x} ${ly}h20v20h-20z"/>`;
    svg += `<text x="${x + 28}" y="${ly + 18}" fill="#333" font-size="18">${escXml(names[item.key])}</text>`;
  });
  return svg;
};

// Get data for specific epsilon and data size
// 获取特定 epsilon 和数据大小的数据
const getData = (results, dataSize, epsilon) => {
  const algos = [
    "jdb_pgm",
    "external_pgm",
    "hashmap",
    "binary_search",
    "btreemap",
  ];
  return algos.map((algo) => {
    const r = results.find(
      (r) =>
        r.data_size === dataSize &&
        r.algorithm === algo &&
        (algo === "jdb_pgm" || algo === "external_pgm"
          ? r.epsilon === epsilon
          : true),
    );
    return r || null;
  });
};

// Get accuracy data for specific data size
// 获取特定数据大小的精度数据
const getAccuracyData = (results, dataSize, epsilon) => {
  const jdb = results.find(
    (r) =>
      r.data_size === dataSize &&
      r.algorithm === "jdb_pgm" &&
      r.epsilon === epsilon,
  );
  const ext = results.find(
    (r) =>
      r.data_size === dataSize &&
      r.algorithm === "external_pgm" &&
      r.epsilon === epsilon,
  );
  return { jdb, ext };
};

// Generate throughput chart
// 生成吞吐量图表
const genThroughputChart = (data, baseY, names, epsilon, lang) => {
  const h = CHART_H;
  const maxThroughput = Math.max(
    ...data.filter(Boolean).map((r) => r.throughput / 1e6),
  );
  let svg = genYAxis(baseY, h, maxThroughput, "M/s");
  svg += genYLabel(
    baseY,
    h,
    lang === "en" ? "Throughput (M/s)" : "吞吐量 (M/s)",
  );

  let x = M.l + BAR_GAP;
  const algos = [
    "jdb_pgm",
    "external_pgm",
    "hashmap",
    "binary_search",
    "btreemap",
  ];
  data.forEach((r, i) => {
    if (r) {
      const val = r.throughput / 1e6;
      const subLabel = r.epsilon ? `e=${r.epsilon}` : null;
      svg += genBar(
        x,
        baseY,
        h,
        val,
        maxThroughput,
        getColor(algos[i]),
        names[algos[i]],
        subLabel,
      );
    }
    x += BAR_W + BAR_GAP;
  });
  return svg;
};

// Generate memory chart
// 生成内存图表
const genMemoryChart = (data, baseY, names, title, lang) => {
  const h = CHART_H;
  const maxMem = Math.max(
    ...data.filter(Boolean).map((r) => r.memory_bytes / (1024 * 1024)),
  );
  let svg = `<text x="${W / 2}" y="${baseY - 20}" fill="#333" font-size="20" font-weight="bold" text-anchor="middle">${escXml(title)}</text>`;
  svg += genYAxis(baseY, h, maxMem, "MB");
  svg += genYLabel(baseY, h, lang === "en" ? "Memory (MB)" : "内存 (MB)");

  let x = M.l + BAR_GAP;
  const algos = [
    "jdb_pgm",
    "external_pgm",
    "hashmap",
    "binary_search",
    "btreemap",
  ];
  data.forEach((r, i) => {
    if (r) {
      const val = r.memory_bytes / (1024 * 1024);
      const subLabel = r.epsilon ? `e=${r.epsilon}` : null;
      svg += genBar(
        x,
        baseY,
        h,
        val,
        maxMem,
        getColor(algos[i]),
        names[algos[i]],
        subLabel,
      );
    }
    x += BAR_W + BAR_GAP;
  });
  return svg;
};

// Generate accuracy chart (avg only with epsilon reference)
// 生成精度对比图表（仅平均误差，带 epsilon 参考线）
const genAccuracyChart = (accData, baseY, names, title, epsilon, lang) => {
  const h = CHART_H;
  const { jdb, ext } = accData;
  if (!jdb && !ext) return "";

  const maxError =
    Math.max(epsilon, jdb?.avg_error || 0, ext?.avg_error || 0) * 1.2;
  let svg = `<text x="${W / 2}" y="${baseY - 20}" fill="#333" font-size="20" font-weight="bold" text-anchor="middle">${escXml(title)}</text>`;
  svg += genYAxis(baseY, h, maxError, "");
  svg += genYLabel(baseY, h, lang === "en" ? "Avg Error" : "平均误差");

  const barData = [
    {
      label:
        lang === "en" ? `Epsilon (e=${epsilon})` : `配置精度 (e=${epsilon})`,
      val: epsilon,
      color: "#94a3b8",
    },
    {
      label: `${names.jdb_pgm}`,
      val: jdb?.avg_error || 0,
      color: ALGORITHM_COLORS.jdb_pgm,
    },
    {
      label: `${names.external_pgm}`,
      val: ext?.avg_error || 0,
      color: ALGORITHM_COLORS.external_pgm,
    },
  ];

  let x = M.l + BAR_GAP + 60;
  barData.forEach((item) => {
    const barH = maxError > 0 ? (item.val / maxError) * h : 0;
    const y = baseY + h - barH;
    if (barH > 0) {
      svg += `<path fill="${item.color}" d="M${x} ${y}h${BAR_W}v${barH}h-${BAR_W}z"/>`;
    }
    svg += `<text x="${x + BAR_W / 2}" y="${y - 8}" fill="#333" font-size="12" text-anchor="middle">${item.val.toFixed(2)}</text>`;
    svg += `<text x="${x + BAR_W / 2}" y="${baseY + h + 20}" fill="#333" font-size="12" text-anchor="end" transform="rotate(-45 ${x + BAR_W / 2} ${baseY + h + 20})">${escXml(item.label)}</text>`;
    x += BAR_W + 60;
  });
  return svg;
};

// Generate full SVG
// 生成完整 SVG
const genSvg = (benchData, accuracyData, lang) => {
  const names = lang === "en" ? ALGORITHM_NAMES : ALGORITHM_NAMES_ZH;
  const dataSize = 1000000;
  const epsilon = 64;
  const data = getData(benchData.results, dataSize, epsilon);
  const accData = getAccuracyData(accuracyData.results, dataSize, epsilon);

  const title = lang === "en" ? "Pgm-Index Benchmark" : "Pgm 索引评测";
  const subtitle =
    lang === "en"
      ? "Throughput and Memory comparison with same epsilon value (e=64)"
      : "相同 epsilon 值 (e=64) 下的吞吐量和内存对比";
  const sizeLabel =
    lang === "en"
      ? `${formatDataSize(dataSize)} Elements`
      : `${formatDataSize(dataSize)} 个元素`;
  const memTitle = lang === "en" ? "Memory Usage Comparison" : "内存使用对比";
  const accTitle =
    lang === "en" ? "Prediction Accuracy Comparison" : "预测精度对比";
  const note =
    lang === "en"
      ? "Precision (e): Lower values provide more accurate predictions but larger index size"
      : "精度 (e): 值越小预测越准确，但索引体积越大";

  // Calculate total height
  // 计算总高度
  const chartGap = 160;
  const totalH =
    M.t + CHART_H + chartGap + CHART_H + chartGap + CHART_H + M.b + 80;

  let svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${W}" height="${totalH}" viewBox="0 0 ${W} ${totalH}">`;
  svg += `<path fill="#fff" d="M0 0h${W}v${totalH}H0z"/>`;

  // Title
  // 标题
  svg += `<text x="${W / 2}" y="80" fill="#222" font-size="46" font-weight="bold" text-anchor="middle">${escXml(title)}</text>`;
  svg += `<text x="${W / 2}" y="116.8" fill="#666" font-size="22" text-anchor="middle">${escXml(subtitle)}</text>`;
  svg += `<text x="${W / 2}" y="${M.t + 26}" fill="#333" font-size="24" font-weight="bold" text-anchor="middle">${escXml(sizeLabel)}</text>`;

  // Throughput chart
  // 吞吐量图表
  const throughputY = M.t + 56;
  svg += genThroughputChart(data, throughputY, names, epsilon, lang);

  // Memory chart
  // 内存图表
  const memoryY = throughputY + CHART_H + chartGap;
  svg += genMemoryChart(data, memoryY, names, memTitle, lang);

  // Accuracy chart
  // 精度图表
  const accuracyY = memoryY + CHART_H + chartGap;
  svg += genAccuracyChart(accData, accuracyY, names, accTitle, epsilon, lang);

  // Note
  // 注释
  const noteY = accuracyY + CHART_H + 80;
  svg += `<text x="${W / 2}" y="${noteY}" fill="#666" font-size="14" text-anchor="middle">${escXml(note)}</text>`;

  // Legend
  // 图例
  svg += genLegend(noteY + 30, names);

  svg += "</svg>";
  return svg;
};

const main = () => {
  const benchData = JSON.parse(readFileSync(BENCH_PATH, "utf8"));
  const accuracyData = JSON.parse(readFileSync(ACCURACY_PATH, "utf8"));

  const enSvg = genSvg(benchData, accuracyData, "en");
  writeFileSync(EN_SVG, enSvg);
  console.log(`Written: ${EN_SVG}`);

  const zhSvg = genSvg(benchData, accuracyData, "zh");
  writeFileSync(ZH_SVG, zhSvg);
  console.log(`Written: ${ZH_SVG}`);
};

main();
