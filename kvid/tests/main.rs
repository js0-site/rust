use std::time::Duration;

use aok::{OK, Void};
use kvid::KvId;
use log::info;

#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
}

pub static KVID_TEST: KvId = KvId::new("test");

#[tokio::test]
async fn test() -> Void {
  xboot::init().await?;
  for i in 0..300 {
    let id = KVID_TEST.next().await?;
    info!("{}", id);
    if i > 5 {
      tokio::time::sleep(Duration::from_secs(1)).await;
    }
  }
  OK
}
