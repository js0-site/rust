#![recursion_limit = "256"]
use aok::{OK, Void};
use mimalloc::MiMalloc;
use rustls::crypto::ring::default_provider;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[static_init::constructor(0)]
extern "C" fn _init() {
  log_init::init();
}

#[tokio::main]
async fn main() -> Void {
  xboot::init().await?;
  let _ = default_provider().install_default();
  smtp_srv::run().await;
  OK
}
