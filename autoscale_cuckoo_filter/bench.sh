#!/bin/bash
set -e

./test.sh
# Run performance comparison benchmarks between different Cuckoo Filter implementations
# 运行不同布谷鸟过滤器实现之间的性能对比基准测试
cargo bench --bench comparison
./table.js
./svg.js
