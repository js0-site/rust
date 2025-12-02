#![cfg_attr(docsrs, feature(doc_cfg))]

mod post;
pub mod record_type;
mod resolve;

#[cfg(feature = "mx")]
#[cfg_attr(docsrs, doc(cfg(feature = "mx")))]
mod mx;

#[cfg(feature = "mx")]
#[cfg_attr(docsrs, doc(cfg(feature = "mx")))]
pub use mx::{Mx, mx};
pub use post::{Answer, DohError, post};
pub use resolve::{DOH_LI, resolve};
