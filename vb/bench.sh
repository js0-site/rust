#!/usr/bin/env bash

set -e
DIR=$(realpath $0) && DIR=${DIR%/*}
cd $DIR
set -x

cargo bench --bench compare --features all
./table.js
./svg.js
