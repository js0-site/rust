#!/usr/bin/env bun

import { writeFileSync } from "node:fs";
import { execSync } from "child_process";
import read from "@3-/read";
import build from "./build.js";

// Read package.json and prepare versions
// 读取 package.json 并准备版本号
const package_path = "package.json",
  pkg_json = JSON.parse(read(package_path)),
  version_parts = pkg_json.version.split("."),
  new_version = version_parts[0] + "." + version_parts[1] + "." + (Number(version_parts[2]) + 1);

pkg_json.version = new_version;

// Write updated package.json to root
// 将新版本号写入根目录 package.json
writeFileSync(package_path, JSON.stringify(pkg_json, null, 2) + "\n");

// Run build function to compile and optimize the package
// 运行 build 函数来编译并优化包
await build();

// Publish to npm
// 发布到 npm
execSync("npm publish --access=public --registry=https://registry.npmjs.org", {
  cwd: "pkg",
  stdio: "inherit",
});
