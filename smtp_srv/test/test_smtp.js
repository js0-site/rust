#!/usr/bin/env bun

import nodemailer from "nodemailer";
import { env } from "node:process";

const { SMTP_USER, SMTP_PASSWORD } = env;

const SMTP = nodemailer.createTransport({
  host: "127.0.0.1",
  port: 465,
  secure: true,
  auth: {
    user: SMTP_USER,
    pass: SMTP_PASSWORD,
  },
  tls: {
    servername: "smtp.js0.site",
  },
});

const send = async (subject) => {
  // const to = '"张三" <zhangsan@js0.site>, xpure@foxmail.com, "迈克" <mike@js0.site>',
  // cc = 'jssite@googlegroups.com, wangwu@js0.site, "抄送-经理" <manager@js0.site>',
  // bcc= '"密送-测试" <monitor@js0.site>, audit@js0.site'
  // const to = "xpure@foxmail.com",
  //   cc = "i18n.site@gmail.com",
  //   bcc = "warnbot@163.com";
  const to = "i18n.site@gmail.com, 4T7fxklQsXQhBPHD@gmail.com",
    cc = "",
    bcc = "";

  const info = await SMTP.sendMail({
    from: '"佚名" <sender@js0.site>',
    to,
    cc,
    bcc,
    subject,
    text: "你好世界",
    html: "<b>你好世界</b>",
  });

  console.log(info);
};

for (let i = 0; ++i < 2; ) {
  await send("测试邮件 " + i + " " + new Date().toISOString());
}
