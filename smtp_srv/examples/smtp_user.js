#!/usr/bin/env bun

import yargs from "yargs";
import { hideBin } from "yargs/helpers";
import redis, { withR } from "./lib/R.js";
import { hashRaw, Algorithm } from "@node-rs/argon2";

const { _ } = yargs(hideBin(process.argv))
    .usage("用法: $0 <邮箱> <密码>")
    .demandCommand(2, "请指定邮箱和密码")
    .help().argv,
  email = _[0].trim().toLowerCase(),
  password = _[1],
  [prefix, domain] = email.split("@"),
  encoder = new TextEncoder(),
  DOMAIN_HOST = encoder.encode("smtpDomainHost:"),
  USER = encoder.encode("smtpUser:"),
  COLON = encoder.encode(":"),
  HOST_ID = "smtpHostId",
  concat = (...arrays) => {
    const res = new Uint8Array(arrays.reduce((acc, cur) => acc + cur.length, 0));
    let pos = 0;
    for (const arr of arrays) {
      res.set(arr, pos);
      pos += arr.length;
    }
    return res;
  },
  write = (val) => {
    if (!val) {
      return new Uint8Array(0);
    }
    const buf = new Uint8Array(8);
    new DataView(buf.buffer).setBigUint64(0, BigInt(val), true);
    let len = 8;
    while (len > 0 && buf[len - 1] === 0) {
      --len;
    }
    return buf.subarray(0, len);
  },
  read = (buf) => {
    if (!buf || buf.length === 0) {
      return 0;
    }
    const padded = new Uint8Array(8);
    padded.set(buf);
    return Number(new DataView(padded.buffer).getBigUint64(0, true));
  };

if (!prefix || !domain) {
  console.error("错误: 无效的邮箱地址: " + email);
  process.exit(1);
}

await withR(async () => {
  const host_key = concat(DOMAIN_HOST, encoder.encode(domain)),
    salt = crypto.getRandomValues(new Uint8Array(16)),
    [host_bin, hash] = await Promise.all([
      redis.getBuffer(Buffer.from(host_key)),
      hashRaw(password, {
        algorithm: Algorithm.Argon2id,
        memoryCost: 65536,
        timeCost: 3,
        parallelism: 1,
        salt,
      }),
    ]);
  const user_key = concat(USER, encoder.encode(domain), COLON, encoder.encode(prefix)),
    val = concat(salt, hash),
    user_set = redis.set(Buffer.from(user_key), Buffer.from(val));
  let host_id;

  if (host_bin) {
    host_id = read(host_bin);
    console.log("域名 " + domain + " 已经映射到 host_id " + host_id);
    await user_set;
  } else {
    const id = await redis.incr(HOST_ID),
      id_bin = write(id);
    await Promise.all([redis.set(Buffer.from(host_key), Buffer.from(id_bin)), user_set]);
    console.log("已自动为域名 " + domain + " 分配新的 host_id " + id);
    host_id = id;
  }

  console.log("成功为 " + email + " 设置密码 (host_id: " + host_id + ")");
});
