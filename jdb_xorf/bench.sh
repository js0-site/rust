#!/usr/bin/env bash

set -e
DIR=$(realpath $0) && DIR=${DIR%/*}
cd $DIR

mkdir -p report

run() {
  cargo bench --features "$1" --no-default-features 2>&1 >"report/$1.txt"
}

run binary-fuse
