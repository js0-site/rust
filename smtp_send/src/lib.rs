mod dkim;
mod error;
mod parse;
mod reject;
mod send;
mod smtp;

use std::sync::Arc;

pub use dkim::signer;
pub use error::{Error, Reject, SendErr};
use mail_send::mail_auth::{
  common::crypto::{RsaKey, Sha256},
  dkim::{DkimSigner, Done},
};
use mail_struct::Mail;
pub use parse::{MAX_RECEIVED, add_received, recv_overflow};
use sk_dkim::Sk;
pub use smtp::Smtp;

pub type Signer = DkimSigner<RsaKey<Sha256>, Done>;
pub type SignerArc = Arc<Signer>;

#[derive(Debug)]
pub struct Send {
  pub selector: String,
  pub sk: Sk,
}

#[derive(Debug, serde::Serialize)]
pub struct SendResult {
  pub error_li: Vec<Error>,
  pub success: usize,
}

// 并发发送邮件并收集结果 / Send emails concurrently and collect results
pub async fn send(mail: &Mail, signer: Option<&Signer>) -> SendResult {
  let (_, results) = unsafe {
    async_scoped::TokioScope::scope_and_collect(|scope| {
      for msg in mail {
        scope.spawn(send::send(msg, signer));
      }
    })
    .await
  };

  let mut success = 0;
  let mut error_li = Vec::new();

  for result in results {
    match result {
      Ok(r) => {
        success += r.success;
        error_li.extend(r.error_li);
      }
      Err(_) => {
        // 任务 panic，跳过此结果 / Task panicked, skip this result
        continue;
      }
    }
  }
  let result = SendResult { error_li, success };

  if !result.error_li.is_empty() && reject::should_reject(&mail.body) {
    reject::reject(mail, &result, signer).await;
  }
  result
}

impl Send {
  pub fn new(selector: impl Into<String>, sk: impl AsRef<[u8]>) -> Self {
    Self {
      selector: selector.into(),
      sk: Sk::new(sk),
    }
  }

  pub async fn send(&self, mail: &mut Mail) -> SendResult {
    // Received 头部过多，拒绝投递 / too many Received headers, reject delivery
    if recv_overflow(&mail.body) {
      return SendResult {
        success: 0,
        error_li: vec![Error::TooManyReceived(format!(
          "{}@{}",
          mail.sender_user, mail.sender_host
        ))],
      };
    }

    let host = &mail.sender_host;
    let Some(signer) = dkim::signer(&self.selector, host, &self.sk) else {
      return SendResult {
        success: 0,
        error_li: vec![Error::DkimInit(host.clone())],
      };
    };

    add_received(&mut mail.body, host, host);
    send(mail, Some(&signer)).await
  }
}
