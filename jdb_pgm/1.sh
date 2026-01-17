#!/usr/bin/env bash

set -e
DIR=$(realpath $0) && DIR=${DIR%/*}
cd $DIR
set -x

[ "$UID" -eq 0 ] || exec sudo "$0" "$@"
