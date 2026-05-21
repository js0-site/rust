#!/usr/bin/env bash

set -e
DIR=$(realpath $0) && DIR=${DIR%/*}
cd $DIR/..

echo "运行 u64_2 性能评测..."
echo "================================"

# 确保依赖已安装
echo "检查依赖..."
cargo check --benches

echo ""
echo "运行完整评测套件..."
echo "================================"

# 运行所有评测
cargo bench --bench u64_encode_decode

echo ""
echo "评测完成！"
echo "================================"
echo "如需查看详细报告，请运行:"
echo "  cargo bench --bench u64_encode_decode -- --output-format html > benchmark_report.html"