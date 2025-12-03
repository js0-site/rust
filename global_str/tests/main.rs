use aok::{OK, Void};
use log::info;
use global_str::GlobalStr;

pub static HOST: GlobalStr = global_str::GlobalStr::new();

#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
  unsafe {
    *HOST.0.get() = Box::leak("test".to_string().into_boxed_str());
  }
}

#[test]
fn test() -> Void {
  info!("> {}", HOST);
  OK
}
