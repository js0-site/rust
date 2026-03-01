#!/bin/bash
set -e
DIR=$(realpath $0) && DIR=${DIR%/*}
cd $DIR

cargo bench --bench bench --features bench-all
./benches/table.js
./benches/svg.js
