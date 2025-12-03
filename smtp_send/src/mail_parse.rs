use aok::{OK, Void};
use log::info;
use mail_struct::Mail;
use msgq::Kv;
use sk_dkim::Sk;

use crate::send::send;

pub struct MailParse {
  pub(crate) selector: String,
  pub(crate) sk: Sk,
}

impl msgq::Parse for MailParse {
  async fn run(&self, kv: &Kv, retry: u64) -> Void {
    for (user_id, mail) in kv {
      let user_id = intbin::bin_u64(user_id);
      let mail: Mail = bitcode::decode(mail)?;
      if let Some((_, domain)) = mail.sender.split_once("@") {
        let domain = domain.to_string();
        info!("{user_id} {} → {}", mail.sender, mail.to_li.join(" / "));
        send(mail, retry, &self.selector, &self.sk, &domain).await?;
      }
    }
    OK
  }

  async fn on_error(&self, kv: Kv, error: String) -> Void {
    for (user_id, mail) in kv {
      let user_id = intbin::bin_u64(user_id);
      let mail: Mail = bitcode::decode(&mail)?;
      info!(
        "{error} {user_id} {} → {}",
        mail.sender,
        mail.to_li.join(" / ")
      );
    }
    OK
  }
}
