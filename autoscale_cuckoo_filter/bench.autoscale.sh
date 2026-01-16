#!/bin/bash

set -e
DIR=$(realpath $0) && DIR=${DIR%/*}
cd $DIR
set -x

# Benchmark only autoscale_cuckoo_filter (for dev/debug)
# 仅测试 autoscale_cuckoo_filter（用于开发调试）
./test.sh
cargo bench --bench comparison --features bench_autoscale
./table.js
