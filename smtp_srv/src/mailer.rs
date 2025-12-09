use aok::{OK, Void};
use mail_struct::UserMail;
use smtp_send::Send;

pub struct Mailer;

genv::s!(DKIM_SK: String);
genv::s!(DKIM_PREFIX: String);

#[static_init::dynamic]
static SMTP: Send = Send::new(&*DKIM_PREFIX, &*DKIM_SK);

impl smtp_recv::Mailer for Mailer {
  async fn send(&self, UserMail { user_id, mail }: UserMail) -> Void {
    log::info!("user_id {}", user_id);
    // let user_id = intbin::to_bin(user_id);
    let _ = SMTP.send(&mail).await;
    OK
  }
}
