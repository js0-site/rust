use aok::Void;
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[static_init::constructor(0)]
extern "C" fn _init() {
  log_init::init();
}

#[tokio::main]
async fn main() -> Void {
  let _ = rustls::crypto::ring::default_provider().install_default();

  xboot::init().await?;
  let smtp_send = smtp_send::SmtpSend::default();
  // loop {
  #[cfg(feature = "jiff")]
  println!("{}", jiff::Zoned::now().strftime("%Y-%m-%d %H:%M:%S"));
  xerr::log!(smtp_send.run().await);
  // }
  Ok(())
}
