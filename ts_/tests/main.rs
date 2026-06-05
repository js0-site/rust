use aok::{OK, Void};
use log::info;

#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
}

#[test]
fn test() -> Void {
  #[cfg(feature = "nano")]
  info!("{}", ts_::nano());
  info!("{}", ts_::sec());
  OK
}
