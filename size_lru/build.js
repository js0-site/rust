#!/usr/bin/env bun

import { copyFileSync, writeFileSync } from "node:fs";
import { homedir } from "node:os";
import { join } from "node:path";
import { $ } from "zx";
import make from "@3-/mdt/make.js";
import read from "@3-/read";

const build = async () => {
  const dir = import.meta.dirname;

  // Set path for rustup components and run wasm-pack build
  // 设置 rustup 组件的 PATH 并运行 wasm-pack 构建
  process.env.PATH = join(homedir(), ".cargo/bin") + ":" + process.env.PATH;
  await $`wasm-pack build --target web --out-name index -- --features wasm`;

  // Optimize WebAssembly binary size and speed via wasm-opt
  // 使用 wasm-opt 优化 WebAssembly 二进制大小与运行速度
  await $`node_modules/wasm-opt/bin/wasm-opt -O3 pkg/index_bg.wasm -o pkg/index_bg.wasm`;

  // Build README.md from mdt template
  // 从 mdt 模板构建 README.md
  await make(dir);

  // Copy compiled JS readme to pkg directory
  // 将编译好的 JS readme 复制到 pkg 目录
  copyFileSync(join(dir, "readme/js.md"), join(dir, "pkg/README.md"));

  // Read package.json
  // 读取 package.json
  const package_path = "package.json",
    pkg_json = JSON.parse(read(package_path));

  // Remove devDependencies and write clean package.json to pkg directory
  // 移除开发依赖并将干净的 package.json 写入 pkg 目录
  delete pkg_json.devDependencies;
  writeFileSync(join("pkg", "package.json"), JSON.stringify(pkg_json, null, 2) + "\n");
};

if (import.meta.main) {
  await build();
}

export default build;
