#!/usr/bin/env bash

set -e
DIR=$(realpath $0) && DIR=${DIR%/*}
cd $DIR
set -x

CRATE=~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/compio-net-0.11.0
rm -rf $CRATE
cargo clean
cargo build
cargo build
