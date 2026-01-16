#!/usr/bin/env coffee

> @3-/redis_lua
  ../../conf/env/kvrocks/IOREDIS.js

await RedisLua(
  import.meta.dirname
  IOREDIS
  'smtp'
)

process.exit()
