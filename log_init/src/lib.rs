#![cfg_attr(docsrs, feature(doc_cfg))]

mod kv;
pub mod layout;

#[cfg(all(target_os = "linux", feature = "systemd"))]
use std::env;

use jiff::tz::TimeZone;
use kv::Kv;
pub use layout::level_color;
use logforth::{append, filter::env_filter::EnvFilterBuilder, starter_log};

#[static_init::dynamic]
pub static TZ: TimeZone = TimeZone::try_system().unwrap();

pub fn init() {
  // Check if we're in a systemd environment (Linux systems with INVOCATION_ID)
  #[cfg(all(target_os = "linux", feature = "systemd"))]
  if env::var("INVOCATION_ID").is_ok() {
    // journald is inherently unbuffered (uses UnixDatagram which doesn't buffer)
    if let Ok(journald) = logforth_append_journald::Journald::new() {
      starter_log::builder()
        .dispatch(|d| {
          d.filter(EnvFilterBuilder::from_default_env().build())
            .append(journald)
        })
        .apply();
      return;
    }
  }
  // If journald fails, fall back to stdout

  // Fallback to stdout logging
  #[cfg(feature = "stdout")]
  {
    let stdout = append::Stdout::default().with_layout(layout::Text::default());

    starter_log::builder()
      .dispatch(|d| {
        d.filter(EnvFilterBuilder::from_default_env().build())
          .append(stdout)
      })
      .apply();
  }
  #[cfg(not(feature = "stdout"))]
  {
    panic!("No logging backend available")
  }
}
