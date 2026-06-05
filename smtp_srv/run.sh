#!/usr/bin/env bash

set -e
DIR=$(realpath $0) && DIR=${DIR%/*}
cd $DIR
export R_NODE="127.0.0.1:6666"
export R_PASSWORD="4wjPHqdrCT181YTv"

set -x

mise exec -- cargo run --all-features -- --nocapture
