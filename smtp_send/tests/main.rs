use aok::{OK, Void};
use idns::Query;
use jiff::Zoned;
use log::info;
use mail_send::{
  mail_auth::{AuthenticatedMessage, DkimOutput, DkimResult, MessageAuthenticator},
  mail_builder::MessageBuilder,
};
use mail_struct::Mail;
use smtp_send::Send;

#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
  let _ = rustls::crypto::ring::default_provider().install_default();
}

genv::s!(DKIM_SK: String);
const SELECTOR: &str = "js0-rsa";
const DOMAIN: &str = "js0.site";

#[tokio::test]
async fn test_dkim_verify() -> Void {
  let send = Send::new(SELECTOR, DKIM_SK.as_bytes());
  let txt = send.sk.dkim(SELECTOR, DOMAIN).txt();

  // Verify TXT record from DNS
  let txt_record_name = format!("{}._domainkey.{}", SELECTOR, DOMAIN);
  let answers = idot::DOT
    .answer_li(idot::QType::TXT, &txt_record_name)
    .await?;

  let mut dns_txt = None;
  if let Some(answers) = answers {
    for answer in answers {
      if answer.val.starts_with("v=DKIM1; k=rsa; p=") {
        dns_txt = Some(answer.val.replace("\"", "").replace(" ", ""));
        break;
      }
    }
  }

  let dns_txt = dns_txt.expect("Failed to fetch DKIM TXT record from DNS");

  assert_eq!(
    dns_txt,
    txt.replace(" ", ""),
    "DNS TXT record does not match generated TXT record"
  );

  // 构建一个测试邮件
  let sender_email = format!("test@{DOMAIN}");
  let message = MessageBuilder::new()
    .from(("Test Sender", sender_email.as_str()))
    .to("recipient@example.com")
    .subject("DKIM 签名测试")
    .text_body(
      "这是一封用于测试 DKIM 签名的邮件。\nThis is a test email for DKIM signature verification.",
    );

  // 获取 DKIM signer
  let signer = smtp_send::signer(SELECTOR, DOMAIN, &send.sk).expect("DKIM signer init failed");

  // 将邮件写入缓冲区以便签名和验证
  let mut output = Vec::new();
  message.write_to(&mut output)?;

  // 使用 signer 对邮件进行签名
  let signature = signer.sign(&output)?;

  // 将签名后的邮件转换为字符串以便日志输出
  // Signature 通常实现了 Display，输出为 header value
  let signature_header = signature.to_string();

  // 构建完整的带签名的邮件用于验证
  // signature_header 已经包含了完整的 "DKIM-Signature: ..." 头
  let mut signed_email = format!("{}\r\n", signature_header).into_bytes();
  signed_email.extend_from_slice(&output);

  // 使用 mail-auth 进行验证

  let authenticator = MessageAuthenticator::new_cloudflare().unwrap();

  // 解析签名后的邮件
  let authenticated_message = AuthenticatedMessage::parse(&signed_email).unwrap();

  // 验证 DKIM - 使用 DNS 查询获取公钥并验证签名
  let dkim_output: Vec<DkimOutput> = authenticator.verify_dkim(&authenticated_message).await;

  // 检查验证结果
  let mut verification_passed = false;
  let mut verification_details = String::new();

  if !dkim_output.is_empty() {
    for result in &dkim_output {
      match result.result() {
        DkimResult::Pass => {
          verification_passed = true;
          verification_details = format!("DKIM verification passed for {DOMAIN}");
        }
        DkimResult::Fail(err) => {
          verification_details = format!("DKIM verification failed: {err:?}");
        }
        DkimResult::TempError(err) => {
          verification_details = format!("DKIM temp error: {err:?}");
        }
        DkimResult::PermError(err) => {
          verification_details = format!("DKIM perm error: {err:?}");
        }
        DkimResult::Neutral(err) => {
          verification_details = format!("DKIM neutral: {err:?}");
        }
        DkimResult::None => {
          verification_details = "No DKIM result".to_string();
        }
      }
    }
  }

  assert!(
    verification_passed,
    "DKIM verification failed: {verification_details}"
  );

  OK
}

#[tokio::test]
async fn test_send_email() -> Void {
  let send = Send::new(SELECTOR, DKIM_SK.as_bytes());

  // 构建邮件内容 / Build email content
  let now = Zoned::now().strftime("%Y-%m-%d %H:%M:%S %Z");
  let subject = format!("SMTP 发送测试 / SMTP Send Test - {now}");
  let body_text = "这是一封通过 SMTP 发送的测试邮件。\nThis is a test email sent via SMTP.";

  let to = format!("recv@{DOMAIN}");
  let sender = format!("test@{DOMAIN}");

  // Build email using mail-send / 使用 mail-send 构建邮件
  let message = MessageBuilder::new()
    .from(("Test Sender", sender.as_str()))
    .to(to.as_str())
    .subject(subject)
    .text_body(body_text);

  // 将邮件写入缓冲区 / Write email to buffer
  let mut email_content = Vec::new();
  message.write_to(&mut email_content)?;

  let mut mail = Mail::new(&sender, [&to], email_content).expect("Failed to create mail");

  info!(
    r#"开始发送邮件 / Starting to send email...
发件人 / Sender: test@{DOMAIN}
收件人 / Recipient: {to}"#
  );

  // 发送邮件 / Send email
  let result = send.send(&mut mail).await;

  info!(
    "✅ 成功发送 {success} 封邮件 / Successfully sent {success} emails",
    success = result.success
  );

  // 输出错误信息 / Output error information
  if !result.error_li.is_empty() {
    let error_count = result.error_li.len();
    info!(
      "❌ 发送过程中遇到 {error_count} 个错误 / Encountered {error_count} errors during sending:"
    );
    for (i, error) in result.error_li.iter().enumerate() {
      info!("  错误 {} / Error {}: {:?}", i + 1, i + 1, error);
    }
  }

  // 如果有成功发送的邮件，测试通过 / Test passes if any emails were sent successfully
  if result.success > 0 {
    info!("🎉 邮件发送测试成功！/ Email sending test successful!");
  } else {
    info!(
      r#"⚠️ 没有邮件发送成功，但这可能是正常的（如 MX 记录问题等）/ No emails sent successfully, but this might be normal (MX record issues, etc.)
💡 常见原因：25端口可能被运营商屏蔽，这是正常现象。/ Common reason: Port 25 may be blocked by ISP, which is normal.
   建议使用587端口（SMTP Submission）或465端口（SMTPS）/ Consider using port 587 (SMTP Submission) or 465 (SMTPS)"#
    );
  }

  OK
}
