#!/usr/bin/env bash
set -e
DIR=$(realpath $0) && DIR=${DIR%/*}
cd $DIR

cargo run --release --example cmp > bench.json
bun ./table.js
bun ./svg.js
