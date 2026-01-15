use aok::{OK, Void};
use mail_struct::{Mail, UserMail};
use smtp_send::{Send, send};

#[derive(Clone)]
pub struct Mailer;

genv::s!(DKIM_SK: String);
genv::s!(DKIM_PREFIX: String);

#[static_init::dynamic]
static SMTP: Send = Send::new(&*DKIM_PREFIX, &*DKIM_SK);

impl smtp_recv::Mailer for Mailer {
  async fn send(&self, UserMail { user_id, mut mail }: UserMail) -> Void {
    log::info!("user_id {user_id}");
    // let user_id = intbin::to_bin(user_id);
    let _ = SMTP.send(&mut mail).await;
    OK
  }

  async fn forward(&self, mail: Mail) -> Void {
    let _ = send(&mail, None).await;
    OK
  }
}
