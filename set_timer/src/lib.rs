#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "async")]
pub mod r#async;

#[cfg(feature = "sync")]
pub mod sync;

#[cfg(feature = "async")]
pub use r#async::set_timer_async;
#[cfg(feature = "sync")]
pub use sync::set_timer;
