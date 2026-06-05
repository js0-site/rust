#!/usr/bin/env bun

import nodemailer from "nodemailer";
import yargs from "yargs";
import { hideBin } from "yargs/helpers";
import net from "net";
import dns from "dns/promises";

const { _ } = yargs(hideBin(process.argv))
  .usage("用法: $0 <邮箱> <密码> <收件人> [服务器IP或主机名]")
  .demandCommand(3, "请指定邮箱、密码和收件人")
  .help().argv,
  email = _[0].trim(),
  password = _[1].trim(),
  to_email = _[2].trim(),
  host_input = _[3] ? _[3].trim() : "127.0.0.1";

let target_ip = host_input;
if (net.isIP(host_input) === 0) {
  try {
    const lookup = await dns.lookup(host_input);
    target_ip = lookup.address;
    console.log("主机名 " + host_input + " 已解析为 IP: " + target_ip);
  } catch (error) {
    console.error("❌ 无法解析服务器主机名 " + host_input + ":", error.message);
    process.exit(1);
  }
}

try {
  console.log("正在创建 SMTP 客户端连接...");
  const domain = email.split("@")[1],
    transporter = nodemailer.createTransport({
      host: target_ip,
      port: 465,
      secure: true,
      auth: {
        user: email,
        pass: password,
      },
      tls: {
        rejectUnauthorized: false,
        servername: domain,
      },
    });

  console.log("正在向 " + to_email + " 发送测试邮件...");
  const { messageId: message_id } = await transporter.sendMail({
    from: email,
    to: to_email,
    subject: "SMTP 发信测试 (" + host_input + ") " + new Date().toLocaleString(),
    text: "这是一封通过 Node.js nodemailer 发送的测试邮件。",
    html: "<b>这是一封通过 Node.js nodemailer 发送的测试邮件。</b>",
  });

  console.log("✅ 邮件发送成功！消息 ID: " + message_id);
} catch (error) {
  console.error("❌ 邮件发送失败:", error);
  process.exit(1);
}
