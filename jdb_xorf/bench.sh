#!/bin/bash
set -e
DIR=$(realpath $0) && DIR=${DIR%/*}
cd $DIR
set -x

cargo bench --bench bench --features bench_all -- --nocapture
cd ./benches
./table.js
./svg.js
