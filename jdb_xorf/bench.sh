#!/bin/bash
set -e
DIR=$(realpath $0) && DIR=${DIR%/*}
cd $DIR

cargo bench --bench bench --features bench-all
node benches/table.js
node benches/svg.js
node regress.js
