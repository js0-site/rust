use aok::{OK, Void};
use log::info;

#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
}

#[test]
fn test() -> Void {
  // 测试基本功能 / Test basic functionality
  info!("> test self_cmd::get()");

  let cmd_result = self_cmd::get();
  assert!(cmd_result.is_ok(), "get() should return Ok");

  info!("Command created successfully");
  OK
}
