use aok::{OK, Void};
use mail_struct::UserMail;
use xkv::{
  R,
  fred::{prelude::StreamsInterface, types::streams::XID},
};

pub struct Mailer;

const R_SMTP: &str = "smtp";

impl smtp_recv::Mailer for Mailer {
  async fn send(&self, UserMail { user_id, mail }: UserMail) -> Void {
    let user_id = intbin::to_bin(user_id);
    let mail = bitcode::encode(&mail);
    let _: () = R
      .xadd(R_SMTP, false, None, XID::Auto, (user_id, mail))
      .await?;
    OK
  }
}
