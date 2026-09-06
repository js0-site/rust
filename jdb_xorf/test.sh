#!/usr/bin/env bash

set -e
DIR=$(realpath $0) && DIR=${DIR%/*}
cd $DIR
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-$DIR/target}
set -x
cargo test --all-features -- --nocapture
