use std::time::Duration;

use aok::Result;
use lettre::{
  AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
  transport::smtp::{
    authentication::Credentials,
    client::{Tls, TlsParameters},
  },
};
use mail_parser::MessageParser;
use serde::Deserialize;
use smtp_recv::run;

mod simple_auth;
use simple_auth::SimpleAuth;

#[derive(Debug, Clone, Deserialize)]
pub struct NameAddress {
  pub name: Option<String>,
  pub address: String,
}

impl std::fmt::Display for NameAddress {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
    std::fs::read_to_string("tests/to_send.yml").expect("无法读取 tests/to_send.yml 文件");
  serde_yaml_bw::from_str(&yaml_content).expect("无法解析 tests/to_send.yml 中的 YAML 数据")
}

#[static_init::dynamic]
static mut SENDED: Vec<mail_struct::Mail> = Vec::new();

#[derive(Clone)]
pub struct PrintMailer;

impl smtp_recv::Mailer for PrintMailer {
  async fn send(&self, user_mail: mail_struct::UserMail) -> anyhow::Result<()> {
    let message = MessageParser::default()
      .parse(&user_mail.mail.body)
      .unwrap();

    println!("\n========== Email Received ==========");
    println!("user_id: {}", user_mail.user_id);

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
    if !user_mail.mail.host_user_li.is_empty() {
      println!("RcptTo: {:?}", user_mail.mail.host_user_li);
    }
    if let Some(subject) = message.subject() {
      println!("Subject: {}", subject);
    }
    if let Some(text) = message.body_text(0) {
      println!("TEXT:\n{}", text);
    }

    SENDED.write().push(user_mail.mail);
    Ok(())
  }
}

#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
}

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

#[tokio::test]
async fn test_smtp() -> Result<()> {
  xboot::init().await?;

  let _ = rustls::crypto::ring::default_provider().install_default();

  // Start SMTP server in background
  tokio::spawn(async {
    if let Err(e) = run(8465, SimpleAuth, PrintMailer, CertByHost).await {
      eprintln!("SMTP server error: {}", e);
    }
  });

  // 等待一段时间确保服务器启动
  tokio::time::sleep(Duration::from_millis(500)).await;

  // 清空之前可能存在的邮件
  SENDED.write().clear();

  // Test Lettre client
  test_lettre_client().await?;

  // 等待一段时间确保邮件已处理
  tokio::time::sleep(Duration::from_secs(1)).await;

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
      let to_list: Vec<_> = to.iter().collect();
      assert_eq!(to_list.len(), expected_mail.to.len());
      for (j, expected_to) in expected_mail.to.iter().enumerate() {
        assert_eq!(to_list[j].name.as_deref(), expected_to.name.as_deref());
        assert_eq!(to_list[j].address.as_deref().unwrap(), expected_to.address);
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
    let mut header_rcpts: std::collections::HashSet<String> = std::collections::HashSet::new();
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

  let creds = Credentials::new("testuser".to_string(), "testpass".to_string());

  let smtp_server = "smtp.js0.site";
  let tls_parameters = TlsParameters::builder(smtp_server.into()).build()?;

  println!("构建传输...");
  let mailer = AsyncSmtpTransport::<Tokio1Executor>::from_url("smtps://127.0.0.1:8465")?
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

  Ok(())
}
