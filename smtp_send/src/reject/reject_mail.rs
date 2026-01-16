use log::error;
use mail_parser::MessageParser;
use mail_send::{
  Error as MailErr,
  mail_builder::MessageBuilder,
  smtp::message::{IntoMessage, Message},
};

use super::{create_tar_zstd::create, encode_mail::encode};
use crate::{Mail, SendResult};

const MIME_ZSTD: &str = "application/zstd";
const ATTACH_NAME: &str = "reject.tar.zst";
const MAILER_DAEMON: &str = "Mailer Daemon";
const NO_TITLE: &str = "NoTitle";
const REJECT_PREFIX: &str = "REJECT → ";
const PARSE_FAIL_SUBJECT: &str = "reject : Unable To Parse";

pub struct RejectMail<'a> {
  msg: MessageBuilder<'a>,
  tar: Vec<u8>,
}

impl<'a> IntoMessage<'a> for &'a RejectMail<'a> {
  fn into_message(self) -> Result<Message<'a>, MailErr> {
    let mut msg = self
      .msg
      .clone()
      .attachment(MIME_ZSTD, ATTACH_NAME, self.tar.as_slice())
      .into_message()?;
    // RFC 5321: 空返回路径防止退信循环 / Empty return path prevents bounce loops
    msg.mail_from = "".into();
    Ok(msg)
  }
}

pub fn mail_message<'a>(mail: &'a Mail, result: &SendResult) -> RejectMail<'a> {
  let yml = serde_yaml_bw::to_string(result).unwrap_or_else(|e| e.to_string());
  error!("{yml}");

  let bin = encode(mail, result);
  let addr = format!("{}@{}", mail.sender_user, mail.sender_host);
  let tar = create(bin.as_slice(), &yml).unwrap_or_default();

  let msg = match MessageParser::default().parse(&mail.body) {
    Some(parsed) => {
      let subject = format!("{REJECT_PREFIX}{}", parsed.subject().unwrap_or(NO_TITLE));
      let body = format!(
        "{yml}\n---\n{}",
        parsed
          .body_text(0)
          .or_else(|| parsed.body_html(0))
          .unwrap_or_default()
      );
      builder(&mail.sender_host, &addr)
        .subject(subject)
        .text_body(body)
    }
    None => builder(&mail.sender_host, &addr)
      .subject(PARSE_FAIL_SUBJECT)
      .text_body(yml),
  };

  RejectMail { msg, tar }
}

fn builder<'a>(host: &str, addr: &str) -> MessageBuilder<'a> {
  MessageBuilder::new()
    .from((MAILER_DAEMON.to_string(), format!("mailer-daemon@{host}")))
    .to(addr.to_string())
}
