#!/usr/bin/env bash

set -e
DIR=$(realpath $0) && DIR=${DIR%/*}
cd $DIR
set -a
. ../../conf/env/dkim.env
set +a
set -x

# ../smtp_srv/test/test_smtp.js >/dev/null
mise exec -- cargo run --all-features -- --nocapture
