#!/usr/bin/env bash

# set -e
# DIR=$(realpath $0) && DIR=${DIR%/*}
# cd $DIR
# set -x

CONF_SH=/etc/kvrocks/conf.sh
if [ -f "$CONF_SH" ]; then
  set -a
  . $CONF_SH
  set +a
fi
