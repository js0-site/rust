#!/usr/bin/env bash

DIR=$(realpath $0) && DIR=${DIR%/*}
cd $DIR
set -ea
. ../conf/TIDB.env
set +a
set -x

mysqldump=mariadb-dump

if ! command -v $mysqldump &>/dev/null; then
  if command -v apt-get &>/dev/null; then
    apt-get install -y mariadb-client
  elif command -v brew &>/dev/null; then
    brew install mariadb
  fi
fi

# 避免 github action 暴露 ip
set +x
cmd="mise exec -- $mysqldump \
  --skip-set-charset \
  --no-data --routines --events --triggers \
  --skip-add-drop-table \
  --compact \
  --compress \
  --routines \
  -d $MYSQL_DB"

echo $cmd

NAME=db_$MYSQL_DB.txt

$cmd -h$MYSQL_HOST -P$MYSQL_PORT -u$MYSQL_USER >$NAME.tmp
mv $NAME.tmp $NAME
set -x

mise exec -- ./dump.coffee
