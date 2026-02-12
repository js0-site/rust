use std::future::Future;

use anyhow::Result;
use mail_struct::{Mail, UserMail};

pub trait Mailer: Clone + Send + Sync + 'static {
  fn send(&self, mail: UserMail) -> impl Future<Output = Result<()>> + Send;
  fn forward(&self, mail: Mail) -> impl Future<Output = Result<()>> + Send;
}
