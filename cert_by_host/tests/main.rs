use aok::{OK, Void};
use log::info;

#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
}

#[tokio::test]
async fn test_async() -> Void {
  xboot::init().await?;
  info!("{:?}", cert_by_host::get("js0.site").await?);
  OK
}
