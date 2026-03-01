use aok::{OK, Void};
use log::info;

#[compio::main]
async fn main() -> Void {
  log_init::init();
  // 分发异步任务
  let rx = compio_dispatch::run(|| async {
    info!("> dispatch async");
    OK
  })
  .unwrap();

  let _ = rx.await.unwrap();

  // 分发阻塞任务
  let rx_blocking = compio_dispatch::blocking(|| {
    info!("> dispatch blocking");
    OK
  })
  .unwrap();

  let _ = rx_blocking.await.unwrap();
  OK
}
