#!/usr/bin/env zx

import { readFileSync, writeFileSync } from "fs";
import { join } from "path";
import os from "os";
import { markdownTable } from "markdown-table";
import { Eta } from "eta";
import resolve from "./benches/i18n/resolve.js";
import {
  fmtTime,
  fmtThroughput,
  groupByDataSize,
  getSortedDataSizes,
  formatDataSize,
  formatMemory,
} from "./benches/js/lib.js";

const ROOT = import.meta.dirname,
  JSON_PATH = join(ROOT, "bench.json"),
  ACC_PATH = join(ROOT, "accuracy.json"),
  BT_PATH = join(ROOT, "build_time.json"),
  EN_MD = join(ROOT, "readme/en.bench.md"),
  ZH_MD = join(ROOT, "readme/zh.bench.md"),
  ETA = new Eta({ autoEscape: false, varName: "_" }),
  LANG_DATA = (await import(resolve("table", "js"))).default;

const getSystemInfo = async () => {
  const cpus = os.cpus(),
    cpu = cpus[0]?.model || "Unknown",
    cores = cpus.length,
    mem = (os.totalmem() / 1024 / 1024 / 1024).toFixed(1),
    platform = os.platform(),
    arch = os.arch(),
    release = os.release(),
    rust_ver = (await $`rustc --version`).stdout.trim();

  let os_name = `${platform} ${release}`;
  if (platform === "darwin") os_name = `macOS ${(await $`sw_vers -productVersion`).stdout.trim()}`;

  return { cpu, cores, mem, osName: os_name, arch, rustVer: rust_ver };
};

const printConfig = (config, lang) => {
  console.log(`${lang.config}:
  ${lang.query_count}: ${config.query_count}
  ${lang.data_sizes}: ${config.data_sizes.join(", ")}
  ${lang.epsilon_values}: ${config.epsilon_values.join(", ")}`);
};

const printConsoleTable = (results, accuracy_data, build_time_data, lang) => {
  const grouped = groupByDataSize(results);

  for (const [data_size, group_results] of getSortedDataSizes(grouped)) {
    console.log(`\n${lang.data_size}: ${formatDataSize(data_size)}`);
    const sorted = group_results.sort((a, b) => b.throughput - a.throughput),
      metrics = {
        [lang.mean_time]: {},
        [lang.throughput]: {},
        [lang.memory]: {},
      };

    for (const r of sorted) {
      const eps = r.epsilon !== undefined ? ` (ε=${r.epsilon})` : "",
        name = lang.algorithm_names[r.algorithm] + eps;
      metrics[lang.mean_time][name] = fmtTime(r.mean_ns);
      metrics[lang.throughput][name] = fmtThroughput(r.throughput);
      metrics[lang.memory][name] = r.memory_bytes > 0 ? formatMemory(r.memory_bytes) : "-";
    }

    for (const [m, v] of Object.entries(metrics)) {
      console.log(`${m}:`);
      for (const [l, val] of Object.entries(v)) console.log(`  ${l}: ${val}`);
    }
  }

  console.log("\n" + "=".repeat(40));
  console.log(lang.accuracy_comparison);
  console.log("=".repeat(40));

  const acc_grouped = {};
  for (const r of accuracy_data.results) {
    const key = `${r.data_size}_eps_${r.epsilon}`;
    if (!acc_grouped[key])
      acc_grouped[key] = {
        data_size: r.data_size,
        epsilon: r.epsilon,
        libs: {},
      };
    acc_grouped[key].libs[r.algorithm] = r;
  }

  for (const key of Object.keys(acc_grouped).sort()) {
    const { data_size, epsilon, libs } = acc_grouped[key];
    console.log(`\n${lang.data_size}: ${formatDataSize(data_size)} (ε=${epsilon})`);
    const metrics = { [lang.max_error]: {}, [lang.avg_error]: {} };
    for (const [algo, r] of Object.entries(libs)) {
      const name = lang.algorithm_names[algo] || algo;
      metrics[lang.max_error][name] = r.max_error ?? "N/A";
      metrics[lang.avg_error][name] = r.avg_error?.toFixed(2) ?? "N/A";
    }
    for (const [m, v] of Object.entries(metrics)) {
      console.log(`${m}:`);
      for (const [l, val] of Object.entries(v)) console.log(`  ${l}: ${val}`);
    }
  }

  console.log("\n" + "=".repeat(40));
  console.log(lang.build_time_comparison);
  console.log("=".repeat(40));

  const bt_grouped = {};
  for (const r of build_time_data.results) {
    const key = `${r.data_size}_eps_${r.epsilon}`;
    if (!bt_grouped[key])
      bt_grouped[key] = {
        data_size: r.data_size,
        epsilon: r.epsilon,
        libs: {},
      };
    bt_grouped[key].libs[r.algorithm] = r;
  }

  for (const key of Object.keys(bt_grouped).sort()) {
    const { data_size, epsilon, libs } = bt_grouped[key];
    console.log(`\n${lang.data_size}: ${formatDataSize(data_size)} (ε=${epsilon})`);
    const metrics = { [lang.build_time]: {} },
      jdb = libs["jdb_pgm"],
      ext = libs["external_pgm"];
    if (jdb) metrics[lang.build_time]["jdb_pgm"] = fmtTime(jdb.build_time_ns);
    if (ext) metrics[lang.build_time]["pgm_index"] = fmtTime(ext.build_time_ns);
    if (jdb && ext && ext.build_time_ns > 0)
      metrics[lang.speedup] = {
        [lang.speedup_label]: (ext.build_time_ns / jdb.build_time_ns).toFixed(2) + "x",
      };
    for (const [m, v] of Object.entries(metrics)) {
      console.log(`${m}:`);
      for (const [l, val] of Object.entries(v)) console.log(`  ${l}: ${val}`);
    }
  }
};

const genMd = async (data, accuracy_data, build_time_data, sys, lang_code) => {
  const lang = (await import(resolve("table", "js", lang_code))).default,
    tpl = readFileSync(resolve("table", "md", lang_code), "utf8"),
    grouped = groupByDataSize(data.results);

  let perf_tables = "";
  for (const [size, results] of getSortedDataSizes(grouped)) {
    if (parseInt(size) !== 1000000) continue;
    const rows = results
      .sort((a, b) => b.throughput - a.throughput)
      .map((r) => [
        lang.algorithm_names[r.algorithm],
        r.epsilon !== undefined ? r.epsilon : "N/A",
        fmtTime(r.mean_ns),
        fmtTime(r.std_dev_ns),
        fmtThroughput(r.throughput),
        r.memory_bytes > 0 ? formatMemory(r.memory_bytes) : "-",
      ]);
    perf_tables += `### ${lang.data_size}: ${formatDataSize(size)}\n\n${markdownTable([
      [lang.algorithm, lang.epsilon, lang.mean_time, lang.std_dev, lang.throughput, lang.memory],
      ...rows,
    ])}\n\n`;
  }

  const acc_results = accuracy_data.results.filter((r) => parseInt(r.data_size) === 1000000),
    acc_map = {};
  for (const r of acc_results) {
    if (!acc_map[r.epsilon]) acc_map[r.epsilon] = {};
    acc_map[r.epsilon][r.algorithm] = r;
  }
  const acc_rows = Object.entries(acc_map)
    .sort((a, b) => a[0] - b[0])
    .map(([eps, libs]) => [
      formatDataSize(1000000),
      eps,
      libs.jdb_pgm?.max_error ?? "N/A",
      libs.jdb_pgm?.avg_error?.toFixed(2) ?? "N/A",
      libs.external_pgm?.max_error ?? "N/A",
      libs.external_pgm?.avg_error?.toFixed(2) ?? "N/A",
    ]);
  const accuracy_table = markdownTable([
    [
      lang.data_size,
      lang.epsilon,
      `${lang.algorithm_names.jdb_pgm} (Max)`,
      `${lang.algorithm_names.jdb_pgm} (Avg)`,
      `${lang.algorithm_names.external_pgm} (Max)`,
      `${lang.algorithm_names.external_pgm} (Avg)`,
    ],
    ...acc_rows,
  ]);

  const bt_results = build_time_data.results.filter((r) => parseInt(r.data_size) === 1000000),
    bt_map = {};
  for (const r of bt_results) {
    if (!bt_map[r.epsilon]) bt_map[r.epsilon] = {};
    bt_map[r.epsilon][r.algorithm] = r;
  }
  const bt_rows = Object.entries(bt_map)
    .sort((a, b) => a[0] - b[0])
    .map(([eps, libs]) => {
      const jdb = libs.jdb_pgm,
        ext = libs.external_pgm;
      return [
        formatDataSize(1000000),
        eps,
        fmtTime(jdb?.build_time_ns || 0),
        fmtTime(ext?.build_time_ns || 0),
        jdb && ext && ext.build_time_ns > 0
          ? (ext.build_time_ns / jdb.build_time_ns).toFixed(2) + "x"
          : "N/A",
      ];
    });
  const build_time_table = markdownTable([
    [
      lang.data_size,
      lang.epsilon,
      `${lang.algorithm_names.jdb_pgm} (Time)`,
      `${lang.algorithm_names.external_pgm} (Time)`,
      lang.speedup,
    ],
    ...bt_rows,
  ]);

  return ETA.renderString(tpl, {
    lang,
    config: data.config,
    sys,
    perf_tables,
    accuracy_table,
    build_time_table,
  });
};

const main = async () => {
  const data = JSON.parse(readFileSync(JSON_PATH, "utf8")),
    acc = JSON.parse(readFileSync(ACC_PATH, "utf8")),
    bt = JSON.parse(readFileSync(BT_PATH, "utf8")),
    sys = await getSystemInfo();

  printConfig(data.config, LANG_DATA);
  printConsoleTable(data.results, acc, bt, LANG_DATA);

  writeFileSync(EN_MD, await genMd(data, acc, bt, sys, "en"));
  writeFileSync(ZH_MD, await genMd(data, acc, bt, sys, "zh"));
};

await main();
