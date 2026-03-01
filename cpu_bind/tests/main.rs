use aok::{OK, Void};
use cpu_bind::{bind, spawn};
use log::info;

#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
}

#[compio::test]
async fn test_bind() -> Void {
  let core_id = bind();
  info!("bound to core {core_id}");
  OK
}

#[compio::test]
async fn test_spawn() -> Void {
  let handle = spawn(|rt| {
    rt.block_on(async {
      info!("running on compio runtime");
      42
    })
  });
  let result = handle.join().expect("thread panicked")?;
  assert_eq!(result, 42);
  info!("spawn result: {result}");
  OK
}
