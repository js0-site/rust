mod reject;
mod smtp;
use mail_send::mail_auth::{
  common::crypto::{RsaKey, Sha256},
  dkim::{DkimSigner, Done},
};
use mail_struct::Mail;
use sk_dkim::Sk;
pub use smtp::Smtp;
mod dkim;
mod error;
mod send;
pub use dkim::signer;
pub use error::Error;

pub type Signer = DkimSigner<RsaKey<Sha256>, Done>;

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

impl Send {
  pub fn new(selector: impl Into<String>, sk: impl AsRef<[u8]>) -> Self {
    Self {
      selector: selector.into(),
      sk: Sk::new(sk),
    }
  }

  pub async fn send(&self, mail: &Mail) -> SendResult {
    let signer = dkim::signer(&self.selector, &mail.sender_host, &self.sk);

    let (_, results) = unsafe {
      async_scoped::TokioScope::scope_and_collect(|scope| {
        for msg in mail {
          scope.spawn(send::send(msg, &signer));
        }
      })
      .await
    };

    let mut success = 0;
    let mut error_li = Vec::new();

    for result in results {
      let r = result.unwrap();
      success += r.success;
      error_li.extend(r.error_li);
    }

    let result = SendResult { error_li, success };

    if !result.error_li.is_empty() {
      reject::reject(mail, &result, &signer).await;
    }

    result
  }
}
