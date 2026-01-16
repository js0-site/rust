#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "nano")]
pub fn nano() -> u64 {
  coarsetime::Clock::now_since_epoch().as_nanos()
}

#[cfg(feature = "sec")]
pub fn sec() -> u64 {
  coarsetime::Clock::now_since_epoch().as_secs()
}

#[cfg(feature = "milli")]
pub fn milli() -> u64 {
  coarsetime::Clock::now_since_epoch().as_millis()
}
