#![cfg_attr(docsrs, feature(doc_cfg))]

pub use signal_hook::consts::{SIGHUP, SIGINT, SIGQUIT, SIGTERM};
use signal_hook_tokio::{Signals, SignalsInfo};

pub const SINGAL_LI: [i32; 4] = [
  SIGTERM, // systemctl stop, kill <pid>, docker stop
  SIGINT,  // Ctrl+C
  SIGQUIT, // Ctrl+\
  SIGHUP,  // systemctl reload
];

pub fn wait(li: impl AsRef<[i32]>) -> SignalsInfo {
  Signals::new(li.as_ref()).expect("Failed to register signal handler")
}

pub fn wait_all() -> SignalsInfo {
  wait(SINGAL_LI)
}
