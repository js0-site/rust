#!/usr/bin/env node
import { writeFileSync, mkdirSync } from "fs";
import { getTargetDir, parseCriterion } from "./lib.js";

const genSvg = (results, lang) => {
  const is_zh = lang === "zh";
  const title = is_zh ? "过滤器性能对比" : "Filter Performance Comparison";
  
  const width = 800;
  const height = 600;
  const margin = { top: 50, right: 30, bottom: 50, left: 60 };
  
  let svg = `<svg width="${width}" height="${height}" xmlns="http://www.w3.org/2000/svg">`;
  svg += `<rect width="${width}" height="${height}" fill="white"/>`;
  svg += `<text x="${width / 2}" y="30" text-anchor="middle" font-size="20" font-weight="bold">${title}</text>`;
  
  const filters = Object.entries(results).flatMap(([lib, filters]) =>
    Object.entries(filters).flatMap(([filter, sizes]) =>
      Object.entries(sizes).map(([size, metrics]) => ({
        name: `${lib}_${filter}_${size}`,
        build: metrics.build?.mean_ns || 0,
        query: metrics.contains?.mean_ns || 0,
      }))
    )
  );
  
  if (filters.length === 0) {
    svg += `<text x="${width / 2}" y="${height / 2}" text-anchor="middle" font-size="16">No benchmark data available</text>`;
    svg += `</svg>`;
    return svg;
  }
  
  const max_build = Math.max(...filters.map((f) => f.build));
  const max_query = Math.max(...filters.map((f) => f.query));
  
  const bar_width = (width - margin.left - margin.right) / filters.length / 2;
  
  filters.forEach((filter, i) => {
    const x = margin.left + i * bar_width * 2;
    const build_height = (filter.build / max_build) * (height - margin.top - margin.bottom);
    const query_height = (filter.query / max_query) * (height - margin.top - margin.bottom);
    
    svg += `<rect x="${x}" y="${height - margin.bottom - build_height}" width="${bar_width * 0.8}" height="${build_height}" fill="#4CAF50"/>`;
    svg += `<rect x="${x + bar_width}" y="${height - margin.bottom - query_height}" width="${bar_width * 0.8}" height="${query_height}" fill="#2196F3"/>`;
  });
  
  svg += `</svg>`;
  return svg;
};

const main = async () => {
  const target_dir = await getTargetDir();
  const results = parseCriterion(target_dir);

  mkdirSync("readme", { recursive: true });

  await Promise.all(
    ["en", "zh"].map((lang) => {
      const svg = genSvg(results, lang);
      const path = `readme/${lang}.bench.svg`;
      writeFileSync(path, svg);
      console.log(`  ${path}`);
    })
  );

  console.log("Generated benchmark SVG charts");
};

main();
