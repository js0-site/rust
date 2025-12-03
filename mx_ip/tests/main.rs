use aok::{OK, Void};
use log::info;
use mx_ip::mx_ip;

#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
}

#[tokio::test]
async fn test_mx_ip() -> Void {
  let host = "gmail.com";
  info!("resolving mx for {}", host);
  let res = mx_ip(host).await?;
  info!("{} mx ips: {:?}", host, res);
  assert!(!res.v4_li.is_empty() || !res.v6_li.is_empty());
  OK
}
