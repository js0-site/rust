use aok::{OK, Void};
use log::info;

#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
}

#[test]
fn test() -> Void {
  let password = "test";
  let (salt, hash) = password_::hash(password);
  info!("{salt:?} {hash:?}");
  assert!(password_::verify(password, salt, hash));
  OK
}
