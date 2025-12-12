#![cfg_attr(docsrs, feature(doc_cfg))]

use futures::stream::StreamExt;
pub use signal_hook::consts::{SIGINT, SIGQUIT, SIGTERM};
use signal_hook_tokio::Signals;

pub async fn restart_signal() -> Result<i32, std::io::Error> {
  let mut signals = Signals::new([
    SIGTERM, // systemctl stop, kill <pid>, docker stop
    SIGINT,  // Ctrl+C
    SIGQUIT, // Ctrl+\
  ])?;

  loop {
    if let Some(signal) = signals.next().await {
      return Ok(signal);
    }
  }
}
