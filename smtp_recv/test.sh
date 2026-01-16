#!/usr/bin/env bash

DIR=$(realpath $0) && DIR=${DIR%/*}
set -e
cd $DIR
set -x

RUST_LOG=trace mise exec -- cargo test --all-features -- --nocapture
