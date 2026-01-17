#!/usr/bin/env bash

set -e
DIR=$(realpath $0) && DIR=${DIR%/*}
cd $DIR
set -x

RUSTFLAGS="-C target-cpu=native" cargo run --example pc --release -F compress -F bitcode
