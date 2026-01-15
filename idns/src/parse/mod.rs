#[cfg(feature = "a")]
pub mod a;

#[cfg(feature = "aaaa")]
pub mod aaaa;

#[cfg(feature = "mx")]
pub mod mx;

#[cfg(feature = "spf")]
pub mod spf;

use crate::{Answer, QType};

pub trait Parse: Sync + Send + 'static {
  const QTYPE: QType;

  fn li(answer: impl IntoIterator<Item = Answer>) -> Vec<Self>
  where
    Self: Sized;
}
