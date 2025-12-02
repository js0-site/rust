use std::sync::atomic::{AtomicU32, Ordering};

use aok::{OK, Void};
use log::info;
use msgq::{Kv, ReadGroup};

#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
}

static N: AtomicU32 = AtomicU32::new(0);

#[derive(Clone)]
struct MailParser;

impl msgq::Parse for MailParser {
  async fn run(&self, mail: &Kv) -> Void {
    info!("print_mail {:?}", mail);
    if N.fetch_add(1, Ordering::SeqCst) % 10 == 9 {
      Err(aok::anyhow!("test error"))
    } else {
      OK
    }
  }

  async fn on_error(&self, mail: Kv, error: String) -> Void {
    info!("on_error {error} {:?}", mail);
    OK
  }
}

#[tokio::test]
async fn test_async() -> Void {
  xboot::init().await?;
  let config = msgq::Conf::new("smtp", "send", "test", 1, 3, 2, 3);
  let read_group = ReadGroup::new(MailParser, config);
  read_group.run().await?;
  OK
}
