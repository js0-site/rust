#!/usr/bin/env bash

set -e
DIR=$(realpath $0) && DIR=${DIR%/*}
cd $DIR
set -a
. ../../conf/env/dkim.env
set +a
set -x

cargo test --all-features -- --nocapture
