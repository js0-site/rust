#!/usr/bin/env bun

import nodemailer from "nodemailer";
import yargs from "yargs";
import { hideBin } from "yargs/helpers";

const { _ } = yargs(hideBin(process.argv))
  .usage("用法: $0 <邮箱> <密码> <收件人> [服务器IP]")
  .demandCommand(3, "请指定邮箱、密码和收件人")
  .help().argv,
  email = _[0].trim(),
  password = _[1].trim(),
  to_email = _[2].trim(),
  host_ip = _[3] ? _[3].trim() : "127.0.0.1";

try {
  console.log("正在创建 SMTP 客户端连接...");
  const domain = email.split("@")[1],
    transporter = nodemailer.createTransport({
      host: host_ip,
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
    subject: "SMTP 发信测试 (" + host_ip + ") " + new Date().toLocaleString(),
    text: "这是一封通过 Node.js nodemailer 发送的测试邮件。",
    html: "<b>这是一封通过 Node.js nodemailer 发送的测试邮件。</b>",
  });

  console.log("✅ 邮件发送成功！消息 ID: " + message_id);
} catch (error) {
  console.error("❌ 邮件发送失败:", error);
  process.exit(1);
}
