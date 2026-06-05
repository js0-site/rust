#!/usr/bin/env bun

import yargs from "yargs";
import { hideBin } from "yargs/helpers";
import { withR } from "./lib/R.js";
import dkim from "./lib/dkim.js";

const { _ } = yargs(hideBin(process.argv))
    .usage("用法: $0 <域名>")
    .demandCommand(1, "请指定域名")
    .help().argv,
  domain = _[0].trim().toLowerCase();

await withR(async () => {
  const [selector, txt_record] = await dkim(domain);
  console.log(
    "\n请配置域名的 DKIM DNS 记录：\n\n" +
      "域名 (主机记录): " +
      selector +
      "._domainkey\n" +
      "记录类型: TXT\n" +
      "记录值: " +
      txt_record
  );
});
