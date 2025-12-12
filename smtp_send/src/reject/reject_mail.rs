use mail_send::{
  Error as MailSendError,
  mail_builder::MessageBuilder,
  smtp::message::{IntoMessage, Message},
};

use super::{create_tar_zstd::create_tar_zstd, encode_mail::encode_mail};
use crate::{Mail, SendResult};

pub struct RejectMail<'a> {
  message: MessageBuilder<'a>,
  tar_zstd: Vec<u8>,
}

impl<'a> IntoMessage<'a> for &'a RejectMail<'a> {
  fn into_message(self) -> Result<Message<'a>, MailSendError> {
    let mut msg = self
      .message
      .clone()
      .attachment(
        "application/zstd",
        "reject.tar.zst",
        self.tar_zstd.as_slice(),
      )
      .into_message()?;
    // Set the envelope sender (MAIL FROM) to empty ("<>") to prevent infinite bounce loops.
    // According to RFC 5321, non-delivery reports (NDRs) must have a null return path
    // so that they do not generate further bounces if they themselves fail to deliver.
    //
    // 将信封发件人 (MAIL FROM) 设置为空 ("<>") 以防止无限退信循环。
    // 根据 RFC 5321，未送达报告 (NDR) 必须具有空返回路径，
    // 以便在它们本身无法发送时不再产生退信。
    msg.mail_from = "".into();
    Ok(msg)
  }
}

pub fn mail_message<'a>(mail: &'a Mail, result: &SendResult) -> RejectMail<'a> {
  let result_yml = match serde_yaml_bw::to_string(result) {
    Ok(s) => s,
    Err(e) => e.to_string(),
  };
  log::error!("{result_yml}");
  let mail_bin = encode_mail(mail, result);
  let address = format!("{}@{}", mail.sender_user, mail.sender_host);
  let tar_zstd = create_tar_zstd(mail_bin.as_slice(), &result_yml);

  let message = if let Some(parsed) = mail_parser::MessageParser::default().parse(&mail.body) {
    let reject_subject = format!("REJECT → {}", parsed.subject().unwrap_or("NoTitle"));
    let new_body_text = format!(
      r#"{result_yml}
---
{}"#,
      parsed
        .body_text(0)
        .map(|s| s.to_string())
        .or_else(|| parsed.body_html(0).map(|s| s.to_string()))
        .unwrap_or_default()
    );

    MessageBuilder::new()
      .from((
        "Mailer Daemon".to_string(),
        format!("mailer-daemon@{}", mail.sender_host),
      ))
      .to(address.clone())
      .subject(reject_subject)
      .text_body(new_body_text)
  } else {
    MessageBuilder::new()
      .from((
        "Mailer Daemon".to_string(),
        format!("mailer-daemon@{}", mail.sender_host),
      ))
      .to(address.clone())
      .subject("reject : Unable To Parse")
      .text_body(result_yml.clone())
  };

  RejectMail { message, tar_zstd }
}
