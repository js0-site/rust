#!/bin/bash

set -e
DIR=$(realpath $0) && DIR=${DIR%/*}
cd $DIR
set -x

cargo criterion -F beach --message-format=json >/tmp/$(dirname)

bun table.js
bun svg.js
