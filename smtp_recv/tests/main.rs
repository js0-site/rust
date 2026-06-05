use std::{fmt, fs, time::Duration};

use aok::Result;
use lettre::{
  AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
  transport::smtp::{
    authentication::Credentials,
    client::{Tls, TlsParameters},
  },
};
use mail_parser::MessageParser;
use rapidhash::HashSetExt;
use rustls::crypto::ring::default_provider;
use serde::Deserialize;
use smtp_recv::run;
use tokio::{
  io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
  net::TcpStream,
  time::{sleep, timeout},
};
use tokio_util::sync::CancellationToken;

mod simple_auth;
use simple_auth::SimpleAuth;

// 简单的转发实现，用于测试
#[derive(Clone)]
struct SimpleForward;

impl mail_forward::Forward for SimpleForward {
  async fn forward(&self, mail: &str) -> anyhow::Result<Option<String>> {
    // Return self to simulate successful forwarding/delivery
    Ok(Some(mail.to_string()))
  }

  async fn forward_set(&self, set: &[String]) -> aok::Result<Vec<String>> {
    let mut res = rapidhash::RapidHashSet::new();
    for s in set {
      res.insert(s.to_string());
    }
    Ok(res.into_iter().collect())
  }
}

#[derive(Debug, Clone, Deserialize)]
pub struct NameAddress {
  pub name: Option<String>,
  pub address: String,
}

impl fmt::Display for NameAddress {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if let Some(name) = &self.name {
      write!(f, "{} <{}>", name, self.address)
    } else {
      write!(f, "{}", self.address)
    }
  }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Attachment {
  pub name: String,
  pub content: Vec<u8>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Mail {
  pub sender: NameAddress,
  pub to: Vec<NameAddress>,
  pub cc: Vec<NameAddress>,
  pub bcc: Vec<String>,
  pub subject: Option<String>,
  pub text: Option<String>,
  pub html: Option<String>,
  pub attachments: Vec<Attachment>,
}

fn to_send_list() -> Vec<Mail> {
  use serde_yaml_bw;
  let yaml_content =
    fs::read_to_string("tests/to_send.yml").expect("无法读取 tests/to_send.yml 文件");
  serde_yaml_bw::from_str(&yaml_content).expect("无法解析 tests/to_send.yml 中的 YAML 数据")
}

#[static_init::dynamic]
static mut SENDED: Vec<mail_struct::Mail> = Vec::new();

#[derive(Clone)]
pub struct PrintMailer;

impl smtp_recv::Mailer for PrintMailer {
  async fn forward(&self, mail: mail_struct::Mail) -> anyhow::Result<()> {
    let message = MessageParser::default().parse(&mail.body).unwrap();

    fn print_addrs(prefix: &str, addrs: &mail_parser::Address) {
      match addrs {
        mail_parser::Address::List(list) => {
          let formatted: Vec<String> = list
            .iter()
            .map(|addr| {
              if let Some(name) = &addr.name {
                format!("{} <{}>", name, addr.address.as_deref().unwrap_or(""))
              } else {
                addr.address.as_deref().unwrap_or("").to_string()
              }
            })
            .collect();
          println!("{}: {}", prefix, formatted.join(", "));
        }
        mail_parser::Address::Group(groups) => {
          println!("{}: (Group) {:?}", prefix, groups);
        }
      }
    }

    if let Some(from) = message.from() {
      print_addrs("From", from);
    }
    if let Some(to) = message.to() {
      print_addrs("To", to);
    }
    if let Some(cc) = message.cc() {
      print_addrs("Cc", cc);
    }
    if !mail.host_user_li.is_empty() {
      println!("RcptTo: {:?}", mail.host_user_li);
    }
    if let Some(subject) = message.subject() {
      println!("Subject: {}", subject);
    }
    if let Some(text) = message.body_text(0) {
      println!("TEXT:\n{}", text);
    }

    SENDED.write().push(mail);
    Ok(())
  }

  async fn send(&self, user_mail: mail_struct::UserMail) -> anyhow::Result<()> {
    println!("\n========== Email Received ==========");
    println!("user_id: {}", user_mail.user_id);
    self.forward(user_mail.mail).await
  }
}

// 日志初始化移到 test_all 函数中

#[derive(Clone)]
struct CertByHost;

impl ssl_trait::CertByHost for CertByHost {
  type Item = cert_by_host::Cert;
  async fn get(&self, host: &str) -> Result<Option<Self::Item>> {
    cert_by_host::CertByHost
      .get(if let Some((_, tld)) = host.split_once(".") {
        tld
      } else {
        host
      })
      .await
  }
}

/// 测试465端口的SMTP发送功能
async fn test_send() -> Result<()> {
  println!("\n=== 测试 465端口 SMTP 发送功能 ===\n");

  let _ = default_provider().install_default();

  // 清空之前可能存在的邮件
  SENDED.write().clear();

  // Test Lettre client
  test_lettre_client().await?;

  // 等待一段时间确保邮件已处理
  sleep(Duration::from_secs(1)).await;

  // 验证 SENDED 中的邮件数量
  let sended = SENDED.read();
  let to_send = to_send_list();
  assert_eq!(
    sended.len(),
    to_send.len(),
    "应该接收到与发送相同数量的邮件"
  );

  // 验证每封邮件的内容
  for (i, mail) in sended.iter().enumerate() {
    // 使用相同的发送邮件对象进行验证
    let expected_mail = &to_send[i];
    let message = MessageParser::default().parse(&mail.body).unwrap();

    // 验证发件人
    let from = message.from().unwrap().first().unwrap();
    assert_eq!(from.name.as_deref(), expected_mail.sender.name.as_deref());
    assert_eq!(
      from.address.as_deref().unwrap(),
      expected_mail.sender.address
    );

    // 验证收件人
    if !expected_mail.to.is_empty() {
      let to = message.to().unwrap();
      let to_li: Vec<_> = to.iter().collect();
      assert_eq!(to_li.len(), expected_mail.to.len());
      for (j, expected_to) in expected_mail.to.iter().enumerate() {
        assert_eq!(to_li[j].name.as_deref(), expected_to.name.as_deref());
        assert_eq!(to_li[j].address.as_deref().unwrap(), expected_to.address);
      }
    }

    // 验证抄送
    if !expected_mail.cc.is_empty() {
      let cc = message.cc().unwrap();
      let cc_list: Vec<_> = cc.iter().collect();
      assert_eq!(cc_list.len(), expected_mail.cc.len());
      for (j, expected_cc) in expected_mail.cc.iter().enumerate() {
        assert_eq!(cc_list[j].name.as_deref(), expected_cc.name.as_deref());
        assert_eq!(cc_list[j].address.as_deref().unwrap(), expected_cc.address);
      }
    }

    // 验证密送 (通过信封收件人)
    // 收集所有 To 和 Cc 地址
    let mut header_rcpts: rapidhash::RapidHashSet<String> = rapidhash::RapidHashSet::new();
    if let Some(to) = message.to() {
      for addr in to.iter() {
        if let Some(a) = &addr.address {
          header_rcpts.insert(a.to_string());
        }
      }
    }
    if let Some(cc) = message.cc() {
      for addr in cc.iter() {
        if let Some(a) = &addr.address {
          header_rcpts.insert(a.to_string());
        }
      }
    }

    // 检查预期的 Bcc 地址是否在 mail.host_user_li 中
    for bcc in &expected_mail.bcc {
      if let Some((user, host)) = xmail::norm_user_host(bcc) {
        assert!(
          mail
            .host_user_li
            .get(&host)
            .is_some_and(|users| users.contains(&user)),
          "Bcc address {} not found in envelope recipients",
          bcc
        );
      }
    }

    // 验证主题
    assert_eq!(message.subject(), expected_mail.subject.as_deref());

    // 验证邮件正文
    if let Some(expected_text) = &expected_mail.text {
      if let Some(text) = message.body_text(0) {
        assert!(
          text.starts_with(expected_text),
          "邮件正文应该以预期文本开头，实际内容: {}",
          text
        );
      } else {
        panic!("邮件正文不应为空");
      }
    }
  }

  println!("验证 SENDED 邮件测试通过，共 {} 封邮件", sended.len());
  Ok(())
}

async fn test_lettre_client() -> Result<()> {
  println!("\n=== 测试 Lettre 客户端连接 ===\n");

  let creds = Credentials::new("sender@example.com".to_string(), "testpass".to_string());

  let smtp_server = "smtp.js0.site";
  let tls_parameters = TlsParameters::builder(smtp_server.into())
    .dangerous_accept_invalid_certs(true)
    .dangerous_accept_invalid_hostnames(true)
    .build()?;

  println!("构建传输...");
  let mailer = AsyncSmtpTransport::<Tokio1Executor>::from_url("smtps://127.0.0.1:465")?
    .credentials(creds)
    .tls(Tls::Wrapper(tls_parameters))
    .build();

  let to_send = to_send_list();

  for (i, mail_data) in to_send.iter().enumerate() {
    // 构建消息，使用 to_send_list 中的数据
    let mut message_builder = Message::builder()
      .from(mail_data.sender.to_string().parse()?)
      .subject(mail_data.subject.as_deref().unwrap_or("无主题"));

    // 添加收件人
    for recipient in &mail_data.to {
      message_builder = message_builder.to(recipient.to_string().parse()?);
    }

    // 添加抄送
    for cc_recipient in &mail_data.cc {
      message_builder = message_builder.cc(cc_recipient.to_string().parse()?);
    }

    // 添加密送
    for bcc_addr in &mail_data.bcc {
      message_builder = message_builder.bcc(bcc_addr.parse()?);
    }

    let body = mail_data.text.as_deref().unwrap_or("");
    let email = message_builder.body(String::from(body))?;

    match mailer.send(email).await {
      Ok(_) => println!("邮件 {} 发送成功!", i + 1),
      Err(e) => {
        eprintln!("无法发送邮件: {:?}", e);
        anyhow::bail!("Lettre 发送失败: {:?}", e);
      }
    }
  }

  // Test rejection logic when sender address does not match the login email
  // 测试发信人地址不匹配时的拒绝逻辑
  let spoofed_email = Message::builder()
    .from("spoofer@example.com".parse()?)
    .to("recipient1@example.com".parse()?)
    .subject("Spoofed Email")
    .body("This should be rejected".to_string())?;

  match mailer.send(spoofed_email).await {
    Ok(_) => {
      anyhow::bail!("应该拒绝不匹配登录邮箱的发信人地址，但发送成功了");
    }
    Err(e) => {
      let err_str = e.to_string();
      println!("预期之内的拒绝: {}", err_str);
      assert!(
        err_str.contains("550") || err_str.contains("Sender address rejected"),
        "应该返回550拒绝错误，但返回了: {}",
        err_str
      );
    }
  }

  Ok(())
}

/// 测试25端口的SMTP转发功能
async fn test_forward() -> Result<()> {
  println!("\n=== 测试 25端口 SMTP 转发功能 ===\n");

  // 测试25端口连接
  test_port_25_connection().await?;

  println!("Port 25 test completed successfully!");
  Ok(())
}

async fn test_port_25_connection() -> Result<()> {
  println!("Testing port 25 connection...");

  // 连接到25端口
  let stream = match TcpStream::connect("127.0.0.1:25").await {
    Ok(stream) => stream,
    Err(e) => {
      eprintln!("Failed to connect to port 25: {}", e);
      return Err(e.into());
    }
  };

  let (reader, mut writer) = stream.into_split();
  let mut reader = BufReader::new(reader);

  // 读取欢迎消息
  let mut line = String::new();
  reader.read_line(&mut line).await?;
  println!("Welcome: {}", line.trim());
  assert!(
    line.starts_with("220"),
    "Expected 220 greeting, got: {}",
    line
  );

  // 发送EHLO
  writer.write_all(b"EHLO test.example.com\r\n").await?;

  // 读取EHLO响应 - 可能是多行响应
  let mut ehlo_response = String::new();
  loop {
    line.clear();
    reader.read_line(&mut line).await?;
    ehlo_response.push_str(&line);
    println!("EHLO line: {}", line.trim());

    // SMTP多行响应：250-xxx 表示还有更多行，250 xxx 表示最后一行
    if line.starts_with("250 ") {
      break;
    }
    if !line.starts_with("250") {
      panic!("Unexpected EHLO response: {}", line);
    }
  }

  println!("Complete EHLO response:\n{}", ehlo_response);
  assert!(
    ehlo_response.contains("STARTTLS"),
    "STARTTLS not advertised"
  );
  assert!(
    ehlo_response.contains("PIPELINING"),
    "PIPELINING not advertised"
  );
  assert!(
    ehlo_response.contains("8BITMIME"),
    "8BITMIME not advertised"
  );

  // 测试基本的SMTP命令序列（不需要认证）

  // MAIL FROM
  writer
    .write_all(b"MAIL FROM:<test@example.com>\r\n")
    .await?;
  line.clear();
  reader.read_line(&mut line).await?;
  println!("MAIL FROM response: {}", line.trim());
  assert!(line.starts_with("250"), "MAIL FROM failed: {}", line);

  // RCPT TO
  writer
    .write_all(b"RCPT TO:<recipient@example.com>\r\n")
    .await?;
  line.clear();
  reader.read_line(&mut line).await?;
  println!("RCPT TO response: {}", line.trim());
  assert!(line.starts_with("250"), "RCPT TO failed: {}", line);

  // DATA
  writer.write_all(b"DATA\r\n").await?;
  line.clear();
  reader.read_line(&mut line).await?;
  println!("DATA response: {}", line.trim());
  assert!(line.starts_with("354"), "DATA failed: {}", line);

  // 发送邮件内容
  writer.write_all(b"Subject: Test Email\r\n").await?;
  writer.write_all(b"From: test@example.com\r\n").await?;
  writer.write_all(b"To: recipient@example.com\r\n").await?;
  writer.write_all(b"\r\n").await?;
  writer
    .write_all(b"This is a test email for port 25.\r\n")
    .await?;
  writer.write_all(b".\r\n").await?;

  // 读取邮件接受响应
  line.clear();
  reader.read_line(&mut line).await?;
  println!("Mail acceptance response: {}", line.trim());
  assert!(line.starts_with("250"), "Mail not accepted: {}", line);

  // QUIT
  writer.write_all(b"QUIT\r\n").await?;
  line.clear();
  reader.read_line(&mut line).await?;
  println!("QUIT response: {}", line.trim());
  assert!(line.starts_with("221"), "QUIT failed: {}", line);

  Ok(())
}

/// 运行所有测试
#[tokio::test]
async fn test_all() -> Result<()> {
  log_init::init();
  xboot::init().await?;

  // Seed the mock certificate for "js0.site" into cert_by_host CACHE
  // 将 "js0.site" 的模拟证书种子写入 cert_by_host CACHE 缓存中
  let key_pem = "-----BEGIN PRIVATE KEY-----\n\
MIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQDHN2OENDrRxIwU\n\
pcZuoj/JPPmLR2U7zGK1Fixj+BGsmCWcnI08vT+SFn/dervFNtAvT38d2QwRJ2cI\n\
YpjhOscBJzz0wKmk06WzkHGSD05v3o1yzISwWUq6gyVqrbt8XJ3KgsXKV760tqdZ\n\
KIyUEw5gmObDxkZSNOAIq2s4O+B7g4BOj38ZabIliPZ3lQEDqy/BxmNu471+Q+Q/\n\
x/oAZ9md8XOmBhVHLRWRKf6DDRRq54lY65savhg6UnDRTLsU08RA7IH3qLE1PXuc\n\
dpZSOuDCRU7No5ISdcTvPbsse167jFKbwHometWvIH6krGQnajPtT//66DOPm4st\n\
bkaGpgVlAgMBAAECggEASO/4EQ64XFXcPbEGKjpgJTMcYEiHAXJOz2MrZHyOKAVa\n\
e/D7avoU/M0dCn9loQp1BWVTMp/lYaBuAi1rMMc+1ibCFBA37D+LIuenFSVif+XZ\n\
HudPS2udxxS+DcVG5/zWzq67cN5YVbysH6IPeiVR9b71ekctNurh/XqZWnDduwpf\n\
Q218xcDiVYWO9RwvASi+64hhV8L+2c5jWRq2VaDQCpowS5pt6opJqmumNwt8yu2b\n\
Mfzl6u3II8buZQkPIdmMDK9t/e6qH6eyMjV+I0Z1LceoUQMgRVxjn30nu7/aBPRr\n\
uQ5WornnE4pwoPxXDCY3CMZUUYRDzLJkXeFmnNxtXwKBgQD/ROrIY5CjPFV+wFx/\n\
Ui+KxvBDywl4sMa8D7W6TCY5oEMQgPh//ri1uThXUM/tdwWdYq7+aJWuvK02g3LJ\n\
UALeDSHDtcj3fQuTZIB6ErBsSIV15+7PquM+4BzMAVY9prlCWub1mESWNHwGqSwW\n\
dbqxMj3Fl5LtJexesHwLRMNYuwKBgQDHyWQvTlsXqnvbl8GKGivE8Qc+2dyDbrfM\n\
UO53HakHNytUOeilfryiU2zF8Us4mwlb2QuKX6KvSDwu5NM59lsqqdymP3ofd6jP\n\
g+mqnS1iK5Fh2th15t/em9MYapUpppTH2P7yxadOKQ460pHI05ADdCT3piciEwGh\n\
G/YNFCLIXwKBgQC/w7Ug3vMcPcn4U2fpWgNPHM4ID7JI0XwqcR7TGjupFMTs1AZa\n\
mS+HjOJPbe0V9lDof+b4RfcUcrco+ay0oP/WckUIMjsL+QtGKbz+d6XDYuOnxJm5\n\
tiXK6S+Y7fQskmAgAgv5Oe9ka380vcfaA41Ban3PP2Pn9ZJRPAGjsX+S2wKBgELt\n\
yb07T7Lu7w1SmoisWcthP58jlwcE2Vf+KNUIv65mIgLWX4TJn2H19rOdMSjmb00w\n\
ufgCL9rupRLEn6qESGhTfVLIYx4VlRznwSjh/OktVUfl66wbyxWlOCOu2QeaAa+t\n\
l0M3SeQaRuUX07TmgxFVIGlCZUu5+ErDtjsNatWbAoGBALowO6ndD7FiBrC80a9g\n\
B/bf+6adXKmFrNwlG6phu4JKdFkwK8vABi2isbdVh4F5tfXUEU7zekum9EQhvUxJ\n\
iTZD+qqQI7NQAxcZajbWurzMUmZ6P83mz7sJd16BeRDuSVsbrCYWSz4al46nTXa4\n\
ClCslnjXQMelhTcZUQXUPZrS\n\
-----END PRIVATE KEY-----";

  let cert_pem = "-----BEGIN CERTIFICATE-----\n\
MIIDETCCAfmgAwIBAgIUMYxc/TcPhTBScoLEkiwOeFupGMIwDQYJKoZIhvcNAQEL\n\
BQAwGDEWMBQGA1UEAwwNc210cC5qczAuc2l0ZTAeFw0yNjA1MjEwMjM3MjBaFw0z\n\
NjA1MTgwMjM3MjBaMBgxFjAUBgNVBAMMDXNtdHAuanMwLnNpdGUwggEiMA0GCSqG\n\
SIb3DQEBAQUAA4IBDwAwggEKAoIBAQDHN2OENDrRxIwUpcZuoj/JPPmLR2U7zGK1\n\
Fixj+BGsmCWcnI08vT+SFn/dervFNtAvT38d2QwRJ2cIYpjhOscBJzz0wKmk06Wz\n\
kHGSD05v3o1yzISwWUq6gyVqrbt8XJ3KgsXKV760tqdZKIyUEw5gmObDxkZSNOAI\n\
q2s4O+B7g4BOj38ZabIliPZ3lQEDqy/BxmNu471+Q+Q/x/oAZ9md8XOmBhVHLRWR\n\
Kf6DDRRq54lY65savhg6UnDRTLsU08RA7IH3qLE1PXucdpZSOuDCRU7No5ISdcTv\n\
Pbsse167jFKbwHometWvIH6krGQnajPtT//66DOPm4stbkaGpgVlAgMBAAGjUzBR\n\
MB0GA1UdDgQWBBTX2ZxKBabN6UjNqpYHi8kGSwWSKjAfBgNVHSMEGDAWgBTX2ZxK\n\
BabN6UjNqpYHi8kGSwWSKjAPBgNVHRMBAf8EBTADAQH/MA0GCSqGSIb3DQEBCwUA\n\
A4IBAQB6Qx6OY9IWBExCrAtfDbasM/RGBVlTkFpSoN93NnCKg4C3uvuxbgOzk0JJ\n\
mlpprdOkOryVTxpKVashzN2IBvutVcUqJj1KKiN/Ei4OvMhowqv54OJzRZsH+XEB\n\
L1YMjIHMaYcN0kZG/bCXNGnQV6Xrr8patFrfPw9SkLz5rNmDWQbTctOM1/j32Kio\n\
e8dJHKbXj9maaNM7K1YdWjUrqZRELbvu+IfmU9QluISYvBb/wIQ1JC9tV8nn5xv1\n\
0fEP04tqJkBnV1+g+Tk+S5zEtczvIjE2XX6HT7XtENJRxByz8Bv+m8558KcFVHMX\n\
jlsBFLm8VZLf/RuaTRMzrw9hcXJ7\n\
-----END CERTIFICATE-----";

  let key = rustls_pemfile::private_key(&mut std::io::Cursor::new(key_pem))
    .unwrap()
    .unwrap();
  let cert = rustls_pemfile::certs(&mut std::io::Cursor::new(cert_pem))
    .map(|c| c.unwrap())
    .collect::<Vec<_>>();

  let ssl_config = std::sync::Arc::new(cert_by_host::SslConfig { key, cert });
  cert_by_host::CACHE
    .pin()
    .insert("js0.site".to_string(), ssl_config);

  // 启动SMTP服务器
  let cancel_token = CancellationToken::new();
  let server_handle = tokio::spawn({
    let cancel_token = cancel_token.clone();
    async move {
      if let Err(e) = run(
        SimpleForward,
        SimpleAuth,
        PrintMailer,
        CertByHost,
        cancel_token,
      )
      .await
      {
        eprintln!("SMTP server error: {}", e);
      }
    }
  });

  // 等待服务器启动
  sleep(Duration::from_millis(1000)).await;

  // 运行测试
  let mut results = Vec::new();

  // 测试465端口发送功能
  println!("\n🚀 开始测试 465端口 SMTP 发送功能...");
  match test_send().await {
    Ok(_) => {
      println!("✅ 465端口测试通过");
      results.push("465端口: 通过");
    }
    Err(e) => {
      eprintln!("❌ 465端口测试失败: {}", e);
      results.push("465端口: 失败");
    }
  }

  // 测试25端口转发功能
  println!("\n🚀 开始测试 25端口 SMTP 转发功能...");
  match test_forward().await {
    Ok(_) => {
      println!("✅ 25端口测试通过");
      results.push("25端口: 通过");
    }
    Err(e) => {
      eprintln!("❌ 25端口测试失败: {}", e);
      results.push("25端口: 失败");
    }
  }

  // 关闭服务器
  cancel_token.cancel();
  let _ = timeout(Duration::from_secs(2), server_handle).await;

  // 输出测试结果总结
  println!("\n 测试结果总结:");
  for result in &results {
    println!("  {}", result);
  }

  // 检查是否所有测试都通过
  let all_passed = results.iter().all(|r| r.contains("通过"));
  if all_passed {
    println!("\n🎉 所有测试都通过了！");
    Ok(())
  } else {
    anyhow::bail!("有测试失败，请检查上面的错误信息");
  }
}
