use aok::{OK, Void};
use log::info;
use sk_dkim::Sk;

#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
}

genv::s!(DKIM_SK: String);

#[test]
fn test() -> Void {
  let sk = Sk::new(DKIM_SK.as_bytes());
  let selector = "rsa";
  let domain = "talkto.me";
  info!(
    r#"

请配置 DKIM 记录
域名 : {selector}._domainkey.{domain}
类型 : TXT
值 : {}
"#,
    sk.dkim(selector, domain).txt()
  );
  OK
}
