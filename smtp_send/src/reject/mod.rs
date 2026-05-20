mod create_tar_zstd;
mod encode_mail;
mod reject_mail;

use idns::mx::CACHE;
use idot::DOT;
use log::error;
use mail_parser::{HeaderName, MessageParser};
use reject_mail::mail_message;

use crate::{Mail, SendResult, Signer, Smtp};

// Auto-Submitted 头名 / Auto-Submitted header name
const AUTO_SUBMITTED: &str = "Auto-Submitted";

/// RFC 3834/5321: 检查是否应发送退信 / Check if bounce should be sent
/// - Auto-Submitted 非 "no" 时不发 / Don't send if Auto-Submitted is not "no"
/// - Return-Path 为空时不发 / Don't send if Return-Path is empty
pub fn should_reject(body: &[u8]) -> bool {
  MessageParser::default()
    .parse(body)
    .map(|m| {
      let auto = m
        .header(AUTO_SUBMITTED)
        .is_some_and(|v| v.as_text().is_none_or(|s| s != "no"));
      let empty_rp = m
        .header(HeaderName::ReturnPath)
        .is_some_and(|v| v.as_text().is_some_and(|s| s.is_empty() || s == "<>"));
      !auto && !empty_rp
    })
    .unwrap_or(true)
}

pub async fn reject(mail: &Mail, result: &SendResult, signer: Option<&Signer>) {
  let msg = mail_message(mail, result);
  let addr = format!("{}@{}", mail.sender_user, mail.sender_host);

  let mx = match CACHE.query(&*DOT, &mail.sender_host).await {
    Ok(r) => r,
    Err(e) => return error!("reject {addr}: {e}"),
  };

  let mx = match mx.as_ref() {
    Some(li) if !li.is_empty() => li,
    _ => return error!("reject {addr}: mx is empty"),
  };

  for m in mx {
    match Smtp::connect(&m.server, signer).await {
      Ok(mut smtp) => match smtp.send(&msg).await {
        Ok(()) => break,
        Err(e) => error!("reject {addr}: {e}"),
      },
      Err(e) => error!("reject {addr} connect to {}: {e}", m.server),
    }
  }
}
