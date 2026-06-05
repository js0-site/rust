#!/usr/bin/env bun

import Cloudflare from "cloudflare";
import yargs from "yargs";
import { hideBin } from "yargs/helpers";
import { CF_TOKEN } from "./conf/CF.js";
import { withR } from "./lib/R.js";
import dkim from "./lib/dkim.js";

const { _ } = yargs(hideBin(process.argv))
    .usage("用法: $0 <域名>")
    .demandCommand(1, "请指定域名")
    .help().argv,
  domain = _[0].trim().toLowerCase(),
  find = async (client, domain) => {
    const zones = await client.zones.list(),
      r = [];
    for await (const zone of zones) {
      const { name } = zone;
      if (domain === name || domain.endsWith("." + name)) {
        r.push(zone);
      }
    }
    return r.sort((a, b) => b.name.length - a.name.length)[0] || null;
  };

await withR(async () => {
  console.log("正在初始化 Cloudflare 客户端...");
  const [selector, txt_record] = await dkim(domain),
    client = new Cloudflare({
      apiToken: CF_TOKEN,
    });

  console.log("正在查找域名 " + domain + " 对应的 Cloudflare Zone...");
  const zone = await find(client, domain);
  if (!zone) {
    console.error("未找到域名 " + domain + " 对应的 Cloudflare Zone，请检查配置。");
    process.exit(1);
  }
  const { name: zone_name, id: zone_id } = zone,
    record_name = selector + "._domainkey." + domain;
  console.log("找到 Zone: " + zone_name + " (" + zone_id + ")");

  console.log("正在获取已有的 " + record_name + " TXT 记录...");
  const dns = client.dns.records,
    records = await dns.list({ zone_id }),
    normalize = (s) => (s || "").replace(/"/g, "").replace(/\s+/g, ""),
    target_normalized = normalize(txt_record);

  for await (const { type, name, id, content } of records) {
    if (type === "TXT" && name === record_name) {
      if (normalize(content) === target_normalized) {
        console.log("✅ 检测到已存在内容相同的记录，无需更新。ID: " + id);
        return;
      }
      console.log("检测到内容不同的记录，正在删除旧记录 ID: " + id);
      await dns.delete(id, { zone_id });
    }
  }

  console.log("正在添加新的 DKIM TXT 记录...");
  const record_content = '"' + txt_record + '"',
    { id: new_id } = await dns.create({
      zone_id,
      type: "TXT",
      name: record_name,
      content: record_content,
      ttl: 1,
    });

  console.log("✅ DKIM TXT 记录配置成功！ID: " + new_id);
});
