use aok::{OK, Void};
use log::info;

#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
}

#[tokio::test]
async fn test_async() -> Void {
  let domain = "gmail.com";

  let r = idoh::mx(domain).await;

  if let Ok(mx_records) = xerr::ok!(r) {
    for mx in mx_records {
      info!(
        "MX Record - Priority: {}, Server: {}, TTL: {}",
        mx.priority, mx.server, mx.ttl
      );
    }
  }

  OK
}
