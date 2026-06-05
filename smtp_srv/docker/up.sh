#!/usr/bin/env bash

set -e
DIR=$(realpath $0) && DIR=${DIR%/*}
cd $DIR
. ../../sh/pid.sh
set -x

exec docker-compose up $@
