#!/usr/bin/env bash

DIR=$(realpath $0) && DIR=${DIR%/*}
set -e
set -a
cd $DIR/../../conf/env/kvrocks
. conf.env
. default.env
set +a
cd $DIR
set -x

cargo test --all-features -- --nocapture
