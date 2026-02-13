#!/usr/bin/env bash

set -e
DIR=$(realpath $0) && DIR=${DIR%/*}
cd $DIR/compio_dns_test

run() {
  export NAME="compio-net $1"
  echo $NAME
  RUSTFLAGS="$2" cargo run --release $3
}

set -x
run "" "" --no-default-features
run "+ compio_dns" "--cfg compio_dns"
