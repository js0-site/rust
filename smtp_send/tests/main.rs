use aok::{OK, Void};
use log::info;
use smtp_send::Send;

#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
}

genv::s!(DKIM_SK: String);

#[tokio::test]
async fn test_dkim_verify() -> Void {
  let selector = "js0-rsa";
  let domain = "js0.site";
  let send = Send::new(selector, DKIM_SK.as_bytes());
  let txt = send.sk.dkim(selector, domain).txt();

  // Verify TXT record from DNS
  let txt_record_name = format!("{}._domainkey.{}", selector, domain);
  info!("Fetching TXT record for: {}", txt_record_name);

  let query = format!("?name={}&type=TXT", txt_record_name);
  let answers = idoh::post("cloudflare-dns.com/dns-query", &query).await?;

  let mut dns_txt = None;
  for answer in answers {
    if answer.data.starts_with("v=DKIM1; k=rsa; p=") {
      dns_txt = Some(answer.data.replace("\"", "").replace(" ", ""));
      break;
    }
  }

  let dns_txt = dns_txt.expect("Failed to fetch DKIM TXT record from DNS");

  assert_eq!(
    dns_txt,
    txt.replace(" ", ""),
    "DNS TXT record does not match generated TXT record"
  );

  // 构建一个测试邮件
  use mail_send::mail_builder::MessageBuilder;
  let sender_email = format!("test@{domain}");
  let message = MessageBuilder::new()
    .from(("Test Sender", sender_email.as_str()))
    .to("recipient@example.com")
    .subject("DKIM 签名测试")
    .text_body(
      "这是一封用于测试 DKIM 签名的邮件。\nThis is a test email for DKIM signature verification.",
    );

  // 获取 DKIM signer
  let signer = smtp_send::signer(selector, domain, &send.sk);

  // 将邮件写入缓冲区以便签名和验证
  let mut output = Vec::new();
  message.write_to(&mut output)?;

  // 使用 signer 对邮件进行签名
  let signature = signer.sign(&output)?;

  // 将签名后的邮件转换为字符串以便日志输出
  // Signature 通常实现了 Display，输出为 header value
  let signature_header = signature.to_string();

  info!("DKIM TXT Record: {}", txt);
  info!("DKIM Signature: {}", signature_header);

  // 构建完整的带签名的邮件用于验证
  // signature_header 已经包含了完整的 "DKIM-Signature: ..." 头
  let mut signed_email = format!("{}\r\n", signature_header).into_bytes();
  signed_email.extend_from_slice(&output);

  // 输出完整的签名邮件用于调试
  info!("Signed Email:\n{}", String::from_utf8_lossy(&signed_email));

  // 使用 mail-auth 进行验证
  use mail_send::mail_auth::{AuthenticatedMessage, DkimOutput, DkimResult, MessageAuthenticator};

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
          verification_details = format!(
            "✅ DKIM 验证通过！\n   域名: {}\n   选择器: {}",
            domain, selector
          );
          info!("{}", verification_details);
        }
        DkimResult::Fail(err) => {
          verification_details = format!("❌ DKIM 验证失败: {:?}", err);
          info!("{}", verification_details);
        }
        DkimResult::TempError(err) => {
          verification_details = format!("⚠️ DKIM 临时错误 (可能是 DNS 问题): {:?}", err);
          info!("{}", verification_details);
        }
        DkimResult::PermError(err) => {
          verification_details = format!("🚫 DKIM 永久错误: {:?}", err);
          info!("{}", verification_details);
        }
        DkimResult::Neutral(err) => {
          verification_details = format!("⚪ DKIM 中立: {:?}", err);
          info!("{}", verification_details);
        }
        DkimResult::None => {
          verification_details = "⚪ 无 DKIM 验证结果".to_string();
          info!("{}", verification_details);
        }
      }
    }
  } else {
    info!("没有找到 DKIM 签名");
  }

  assert!(
    verification_passed,
    "DKIM 签名验证失败！\nTXT 记录: {}\n签名: {}\n详情: {}",
    txt, signature_header, verification_details
  );

  info!(
    "🎉 验证成功！TXT 记录和签名完全匹配！\n   DNS 记录: {selector}._domainkey.{domain}\n   TXT 值: {}",
    txt
  );

  OK
}
