#![cfg_attr(docsrs, feature(doc_cfg))]

mod post;
pub mod record_type;
mod resolve;

#[cfg(feature = "mx")]
pub mod mx;

#[cfg(feature = "mx")]
pub use mx::{Mx, MxLookup};
pub use post::{Answer, DohError, post};
pub use resolve::{DOH_LI, resolve};
mod resolve_trait;
pub use resolve_trait::{Resolve, Resolver};
mod error;
pub use error::{Error, Result};
