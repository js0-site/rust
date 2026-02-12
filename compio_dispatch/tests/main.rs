use aok::{OK, Void};
use log::info;

#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
}

#[compio::test]
async fn test() -> Void {
  let rx = compio_dispatch::DISPATCH
    .dispatch(|| async {
      info!("> dispatch async");
      OK
    })
    .unwrap();
  rx.await.unwrap()
}
